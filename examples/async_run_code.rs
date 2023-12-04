use std::collections::HashMap;

use async_stream::stream;
use futures_core::stream::Stream;
use futures_util::pin_mut;
use futures_util::stream::StreamExt;

use jupyter_client::commands::Command as JupyterCommand;
use jupyter_client::responses::{ExecutionState, IoPubResponse, Response, ShellResponse, Status};
use jupyter_client::Client;

fn jupyter_subscribe_stream() -> impl Stream<Item = Response> {
    // let mut listener = TcpListener::bind(addr).await?;
    let client = Client::existing().expect("creating jupyter connection");
    let receiver = client.iopub_subscribe().unwrap();
    stream! {
        for msg in &receiver {
            yield msg;
        }
    }
}

async fn send_code_to_jupyter(code: &str) {
    let client = Client::existing().expect("creating jupyter connection");
    // send code to jupyter kernel
    let command = JupyterCommand::Execute {
        code: code.into(),
        silent: false,
        store_history: true,
        user_expressions: HashMap::new(),
        allow_stdin: true,
        stop_on_error: false,
    };
    let response = client.send_shell_command(command).expect("sending command");
    if let &Response::Shell(ShellResponse::Execute { ref content, .. }) = &response {
        match content.status {
            Status::Ok | Status::Abort => {
                // debug!("Response: {:#?}", response)
            }
            Status::Error => {
                eprintln!("Error: {}", content.evalue.as_ref().unwrap());
                for line in content.traceback.as_ref().unwrap() {
                    eprintln!("{}", line);
                }
            }
        }
    } else {
        panic!("unexpected response type");
    }
    println!("finished:\n{:#?}", response);
}

#[tokio::main]
async fn main() {
    let stream_response = jupyter_subscribe_stream();
    pin_mut!(stream_response); // needed for iteration

    let code_example0 = String::from("
    import requests

    # Function to get the repository description
    def get_repo_description(url):
        response = requests.get(url)
        description = response.json()['description']
        return description
    
    # Get the repository description
    repo_url = 'https://api.github.com/repos/KillianLucas/open-interpreter'
    description = get_repo_description(repo_url)
    description
    ");
    let code_example1 = String::from("import weasyprint\nweasyprint.HTML('readme.md').write_pdf('readme.pdf')");
    // let code_example2 = String::from("import pandas as pd\nimport numpy as np\nf = pd.DataFrame(np.random.rand(10, 5))\ndisplay(df)"); 
    // Spawn the asynchronous task
    tokio::spawn(async move {
        send_code_to_jupyter(&code_example1).await;
    });

    while let Some(value) = stream_response.next().await {
        println!("got {:#?}", value);
    }
}