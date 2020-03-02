extern crate crypto;

use crypto::digest::Digest;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufStream},
    net::TcpStream,
};

use crate::{errors::Error, util};

pub fn compute_nonce_hash(pass: &str, nonce: &str) -> String {
    let mut digest = crypto::md5::Md5::new();
    digest.input_str(&format!("{}{}", nonce, pass));
    digest.result_str()
}

const TERMCHAR: u8 = 3;

fn verify_rpc_reply_root(root_node: &treexml::Element) -> Result<(), Error> {
    if root_node.name != "boinc_gui_rpc_reply" {
        return Err(Error::DataParseError(
            "invalid response XML root node".to_string(),
        ));
    }
    if root_node.children.is_empty() {
        return Err(Error::DataParseError("Empty response root node".into()));
    }
    Ok(())
}

async fn read_from_boinc_tcpstream(
    conn: &mut BufStream<BufReader<TcpStream>>,
) -> Result<String, Error> {
    let mut recv_buf = Vec::new();
    conn.read_until(TERMCHAR, &mut recv_buf).await?;
    if recv_buf.pop().unwrap_or_default() != TERMCHAR {
        return Err(Error::NetworkError(
            "Unexpected EOF while reading from stream".into(),
        ));
    }

    let s = String::from_utf8(recv_buf)?;
    Ok(s)
}

async fn send_to_boinc_tcpstream(
    conn: &mut BufStream<BufReader<TcpStream>>,
    msg: &str,
) -> Result<(), Error> {
    conn.write_all(msg.as_bytes()).await?;
    conn.write_all(&[TERMCHAR]).await?;
    conn.flush().await?;

    Ok(())
}

pub struct DaemonStream {
    conn: BufStream<BufReader<TcpStream>>,
}

impl DaemonStream {
    pub async fn connect(host: String, password: Option<String>) -> Result<Self, Error> {
        let mut conn = BufStream::new(BufReader::new(TcpStream::connect(host).await?));

        let mut req_root = treexml::Element::new("auth1");

        let mut nonce_sent = false;
        loop {
            let s = format!("{}", &req_root)
                .replace("<?xml version='1.0'?>", "")
                .replace(" />", "/>");

            req_root = treexml::Element::new("boinc_gui_rpc_request");

            send_to_boinc_tcpstream(&mut conn, &s).await?;

            let recv_data = read_from_boinc_tcpstream(&mut conn).await?;

            let root_node = util::parse_node(&recv_data)?;

            for node in root_node.children {
                match &*node.name {
                    "nonce" => {
                        if nonce_sent {
                            return Err(Error::DaemonError(
                                "Daemon requested nonce again - could be a bug".into(),
                            ));
                        }
                        let mut nonce_node = treexml::Element::new("nonce_hash");
                        let pwd = password.clone().ok_or_else(|| {
                            Error::AuthError("Password required for nonce".to_string())
                        })?;
                        nonce_node.text = Some(compute_nonce_hash(
                            &pwd,
                            &node
                                .text
                                .ok_or_else(|| Error::AuthError("Invalid nonce".into()))?,
                        ));

                        let mut auth2_node = treexml::Element::new("auth2");
                        auth2_node.children.push(nonce_node);

                        req_root.children.push(auth2_node);
                        nonce_sent = true;
                    }
                    "unauthorized" => {
                        return Err(Error::AuthError("unauthorized".to_string()));
                    }
                    "error" => {
                        return Err(Error::DaemonError(format!(
                            "BOINC daemon returned error: {:?}",
                            node.text
                        )));
                    }
                    "authorized" => {
                        return Ok(Self { conn });
                    }
                    _ => {
                        return Err(Error::DaemonError(format!(
                            "Invalid response from daemon: {}",
                            node.name
                        )));
                    }
                }
            }
        }
    }

    pub(crate) async fn query(
        &mut self,
        request_data: Vec<treexml::Element>,
    ) -> Result<treexml::Element, Error> {
        if request_data.is_empty() {
            return Err(Error::NullError("Request data cannot be empty".into()));
        }
        let mut req_root = treexml::Element::new("boinc_gui_rpc_request");
        req_root.children = request_data;

        let s = format!("{}", &req_root);
        send_to_boinc_tcpstream(&mut self.conn, &s.replace("<?xml version='1.0'?>", "")).await?;

        let recv_data = read_from_boinc_tcpstream(&mut self.conn)
            .await?
            .replace("<?xml version=\"1.0\" encoding=\"ISO-8859-1\" ?>", "");
        let rsp_root = util::parse_node(&recv_data)?;
        verify_rpc_reply_root(&rsp_root)?;

        Ok(rsp_root)
    }
}
