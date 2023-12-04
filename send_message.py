#!/usr/bin/env python


import zmq
import json
import hmac
import hashlib
import sys
import datetime
import uuid
import pprint
import argparse

PYTHON3 = sys.version_info.major == 3
engine_id = str(uuid.uuid4())
DELIM = b"<IDS|MSG>"

def msg_id():
    return str(uuid.uuid4())

def str_to_bytes(s):
    return s.encode("ascii") if PYTHON3 else bytes(s)

def new_header(msg_type):
    return {
        # "date": datetime.datetime.now().isoformat(),
        "msg_id": msg_id(),
        # "username": "kernel",
        # "session": engine_id,
        "msg_type": msg_type,
        # "version": "5.0",
    }

class Frontend(object):

    context = zmq.Context()

    def __init__(self, filename):
        self.filename = filename

        with open(self.filename) as infile:
            connection_data = json.load(infile)

        self.port = connection_data["shell_port"]
        self.key = str_to_bytes(connection_data["key"])
        self.auth = hmac.HMAC(self.key, digestmod=hashlib.sha256)

        self.socket = self.context.socket(zmq.REQ)
        self.socket.connect("tcp://localhost:{port}".format(port=self.port))

    def get_kernel_info(self):
        self.send("kernel_info_request")
        msg = self.socket.recv_multipart()
        identities, m = self.deserialize_wire_msg(msg)
        return m

    def get_comm_info(self):
        self.send("comm_info_request")
        msg = self.socket.recv_multipart()
        identities, m = self.deserialize_wire_msg(msg)
        return m

    def shutdown(self, restart=False):
        self.send("shutdown_request", content={"restart": restart})
        msg = self.socket.recv_multipart()
        identities, m = self.deserialize_wire_msg(msg)
        return m

    def sign(self, msg_lst):
        h = self.auth.copy()
        for m in msg_lst:
            h.update(m)
        return str_to_bytes(h.hexdigest())

    def send(
        self,
        msg_type,
        content=None,
        parent_header=None,
        metadata=None,
        identities=None,
    ):
        header = new_header(msg_type)
        if content is None:
            content = {}
        if parent_header is None:
            parent_header = {}
        if metadata is None:
            metadata = {}

        def encode(msg):
            return str_to_bytes(json.dumps(msg))

        msg_lst = [
            encode(header),
            encode(parent_header),
            encode(metadata),
            encode(content),
        ]
        signature = self.sign(msg_lst)
        parts = [DELIM, signature, msg_lst[0], msg_lst[1], msg_lst[2], msg_lst[3]]

        if identities:
            parts = identities + parts
        self.socket.send_multipart(parts)

    def deserialize_wire_msg(self, wire_msg):
        delim_idx = wire_msg.index(DELIM)
        identities = wire_msg[:delim_idx]
        m_signature = wire_msg[delim_idx + 1]
        msg_frames = wire_msg[delim_idx + 2 :]

        def decode(msg):
            return json.loads(msg.decode("ascii") if PYTHON3 else msg)

        m = {}
        m["header"] = decode(msg_frames[0])
        m["parent_header"] = decode(msg_frames[1])
        m["metadata"] = decode(msg_frames[2])
        m["content"] = decode(msg_frames[3])
        check_sig = self.sign(msg_frames)
        if check_sig != m_signature:
            raise ValueError("signatures do not match")

        return identities, m

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("filename")
    args = parser.parse_args()

    f = Frontend(args.filename)
    pprint.pprint(f.get_kernel_info())
