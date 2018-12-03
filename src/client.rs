use commands::Command;
use connection_config::ConnectionConfig;
use errors::Result;
use failure::format_err;
use glob::glob;
use hmac::Mac;
use log::{debug, trace};
use paths::jupyter_runtime_dir;
use responses::Response;
use signatures::HmacSha256;
use std::env::current_dir;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use socket::Socket;

fn find_connection_file<S>(glob_pattern: S, paths: Option<Vec<PathBuf>>) -> Option<PathBuf>
where
    S: Into<String>,
{
    let paths = paths.unwrap_or_else(|| {
        vec![
            current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            jupyter_runtime_dir(),
        ]
    });
    trace!("connection file paths to search: {:?}", paths);

    let glob_pattern = glob_pattern.into();

    for path in paths.into_iter() {
        let pattern = path.join(&glob_pattern);
        trace!("glob pattern: {:?}", pattern);
        let matches = glob(pattern.to_str().unwrap()).unwrap();
        let mut matches: Vec<PathBuf> = matches.map(|m| m.unwrap()).collect();
        trace!("matches: {:?}", matches);
        if !matches.is_empty() {
            matches.sort_by_key(|p| {
                let metadata = fs::metadata(p).unwrap();
                metadata.modified().unwrap()
            });
            trace!("sorted matches: {:#?}", matches);
            return Some(matches.last().unwrap().clone());
        }
    }
    None
}

pub struct Client {
    shell_socket: Socket,
    control_socket: Socket,
    iopub_socket: Arc<Mutex<Socket>>,
    heartbeat_socket: Arc<Mutex<Socket>>,
    auth: HmacSha256,
}

impl Client {
    pub fn existing() -> Result<Self> {
        use std::fs::File;

        find_connection_file("kernel-*.json", None)
            .ok_or_else(|| format_err!("no connection file found"))
            .and_then(|filename| {
                debug!("found connection file {:?}", filename);
                let f = File::open(filename)?;
                Self::from_reader(f)
            })
    }

    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: Read,
    {
        let config: ConnectionConfig = ConnectionConfig::from_reader(reader)?;
        let auth = HmacSha256::new_varkey(config.key.as_bytes())
            .map_err(|e| format_err!("Error constructing HMAC: {:?}", e))?;

        let ctx = zmq::Context::new();

        let shell_socket = Socket::new_shell(&ctx, &config)?;
        let control_socket = Socket::new_control(&ctx, &config)?;
        let iopub_socket = Socket::new_iopub(&ctx, &config)?;
        let heartbeat_socket = Socket::new_heartbeat(&ctx, &config)?;

        Ok(Client {
            shell_socket,
            control_socket,
            iopub_socket: Arc::new(Mutex::new(iopub_socket)),
            heartbeat_socket: Arc::new(Mutex::new(heartbeat_socket)),
            auth: auth,
        })
    }

    pub fn send_shell_command(&self, command: Command) -> Result<Response> {
        debug!("Sending shell command: {:?}", command);
        self.send_command_to_socket(command, &self.shell_socket)
    }

    pub fn send_control_command(&self, command: Command) -> Result<Response> {
        debug!("Sending control command: {:?}", command);
        self.send_command_to_socket(command, &self.control_socket)
    }

    fn send_command_to_socket(&self, command: Command, socket: &Socket) -> Result<Response> {
        let wire = command.into_wire(self.auth.clone())?;
        socket.send_wire(wire)?;
        let resp_wire = socket.recv_wire(self.auth.clone())?;
        resp_wire.into_response()
    }

    pub fn iopub_subscribe(&self) -> Result<Receiver<Response>> {
        let (tx, rx) = mpsc::channel();
        let socket = self.iopub_socket.clone();
        let auth = self.auth.clone();

        thread::spawn(move || loop {
            let socket = socket.lock().unwrap();
            let wire = socket.recv_wire(auth.clone()).unwrap();
            let msg = wire.into_response().unwrap();
            tx.send(msg).unwrap();
        });

        Ok(rx)
    }

    pub fn heartbeat_every(&self, seconds: Duration) -> Result<Receiver<()>> {
        let (tx, rx) = mpsc::channel();
        let socket = self.heartbeat_socket.clone();

        thread::spawn(move || loop {
            let socket = socket.lock().unwrap();
            let _msg = socket.heartbeat().unwrap();
            tx.send(()).unwrap();
            thread::sleep(seconds);
        });
        Ok(rx)
    }

    pub fn heartbeat(&self) -> Result<Receiver<()>> {
        self.heartbeat_every(Duration::from_secs(1))
    }
}
