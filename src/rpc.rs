extern crate crypto;
extern crate std;
extern crate treexml;

use rpc::crypto::digest::Digest;
use std::io::Read;
use std::io::Write;

use errors::Error;
use util;

pub fn compute_nonce_hash(pass: &str, nonce: &str) -> String {
    let mut digest = crypto::md5::Md5::new();
    digest.input_str(&format!("{}{}", nonce, pass));
    digest.result_str()
}

const TERMCHAR: u8 = 3;

pub trait DaemonStream {
    fn query(&mut self, Vec<treexml::Element>) -> Result<treexml::Element, Error>;
}

fn read_from_boinc_tcpstream(stream: &mut std::net::TcpStream) -> Result<String, Error> {
    let mut recv_buf = Vec::new();
    for byte in try!(stream.try_clone()).bytes() {
        let data: u8 = try!(byte);
        if data == TERMCHAR {
            break;
        } else {
            recv_buf.push(data);
        }
    }
    let s = try!(String::from_utf8(recv_buf));
    Ok(s)
}

fn send_to_boinc_tcpstream(stream: &mut std::net::TcpStream, msg: &str) -> Result<(), Error> {
    try!(stream.write_all(msg.as_bytes()));
    try!(stream.write_all(&vec![TERMCHAR]));

    Ok(())
}

fn verify_rpc_reply_root(root_node: &treexml::Element) -> Result<(), Error> {
    if root_node.name != "boinc_gui_rpc_reply" {
        return Err(Error::DataParseError("invalid response XML root node".to_string()));
    }
    if root_node.children.is_empty() {
        return Err(Error::DataParseError("Empty response root node".into()));
    }
    Ok(())
}

pub struct SimpleDaemonStream {
    conn: std::net::TcpStream,
}

impl SimpleDaemonStream {
    pub fn connect(host: &std::net::SocketAddr,
                   password: &Option<String>)
                   -> Result<SimpleDaemonStream, Error> {

        let mut stream = try!(std::net::TcpStream::connect(host));

        let mut req_root = treexml::Element::new("auth1");

        let mut nonce_sent = false;
        loop {
            let s =
                format!("{}", &req_root).replace("<?xml version='1.0'?>", "").replace(" />", "/>");

            req_root = treexml::Element::new("boinc_gui_rpc_request");

            try!(send_to_boinc_tcpstream(&mut stream, &s));

            let recv_data = try!(read_from_boinc_tcpstream(&mut stream));

            let root_node = try!(util::parse_node(&recv_data));

            for node in root_node.children {
                match &*node.name {
                    "nonce" => {
                        if nonce_sent {
                            return Err(Error::DaemonError("Daemon requested nonce again - could \
                                                           be a bug"
                                .into()));
                        }
                        let mut nonce_node = treexml::Element::new("nonce_hash");
                        let pwd = try!(password.clone()
                            .ok_or(Error::AuthError("Password required for nonce".to_string())));
                        nonce_node.text = Some(compute_nonce_hash(&pwd,
                                                                  &try!(node.text
                                                        .ok_or(Error::AuthError("Invalid nonce"
                                                            .into())))));

                        let mut auth2_node = treexml::Element::new("auth2");
                        auth2_node.children.push(nonce_node);

                        req_root.children.push(auth2_node);
                        nonce_sent = true;
                    }
                    "unauthorized" => {
                        return Err(Error::AuthError("unauthorized".to_string()));
                    }
                    "error" => {
                        return Err(Error::DaemonError(format!("BOINC daemon returned error: \
                                                               {:?}",
                                                              node.text)));
                    }
                    "authorized" => {
                        return Ok(SimpleDaemonStream { conn: stream });
                    }
                    _ => {
                        return Err(Error::DaemonError(format!("Invalid response from daemon: {}",
                                                              node.name)));
                    }
                }
            }
        }
    }
}

impl DaemonStream for SimpleDaemonStream {
    fn query(&mut self, request_data: Vec<treexml::Element>) -> Result<treexml::Element, Error> {
        if request_data.is_empty() {
            return Err(Error::NullError("Request data cannot be empty".into()));
        }
        let mut req_root = treexml::Element::new("boinc_gui_rpc_request");
        req_root.children = request_data;

        let s = format!("{}", &req_root);
        try!(send_to_boinc_tcpstream(&mut self.conn, &s.replace("<?xml version='1.0'?>", "")));

        let recv_data = try!(read_from_boinc_tcpstream(&mut self.conn))
            .replace("<?xml version=\"1.0\" encoding=\"ISO-8859-1\" ?>", "");
        let mut rsp_root = try!(util::parse_node(&recv_data));
        try!(verify_rpc_reply_root(&mut rsp_root));

        Ok(rsp_root)
    }
}
