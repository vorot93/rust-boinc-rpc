extern crate crypto;

use bytes::BytesMut;
use crypto::digest::Digest;
use encoding::{all::ISO_8859_1, DecoderTrap, EncoderTrap, Encoding};
use futures::SinkExt;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    stream::StreamExt,
};
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::*;

use crate::{errors::Error, util};

fn compute_nonce_hash(pass: &str, nonce: &str) -> String {
    let mut digest = crypto::md5::Md5::new();
    digest.input_str(&format!("{}{}", nonce, pass));
    digest.result_str()
}

const TERMCHAR: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CodecMode {
    Client,
    Server,
}

pub struct BoincCodec {
    mode: CodecMode,
    next_index: usize,
}

impl BoincCodec {
    #[must_use]
    pub const fn new(mode: CodecMode) -> Self {
        Self {
            mode,
            next_index: 0,
        }
    }
}

impl Decoder for BoincCodec {
    type Item = Vec<treexml::Element>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let read_to = src.len();

        if let Some(offset) = src[self.next_index..read_to]
            .iter()
            .position(|b| *b == TERMCHAR)
        {
            let newline_index = offset + self.next_index;
            self.next_index = 0;
            let line = src.split_to(newline_index + 1);
            let line = &line[..line.len() - 1];
            let line = ISO_8859_1
                .decode(line, DecoderTrap::Strict)
                .map_err(|e| Error::DataParseError(format!("Invalid data received: {}", e)))?;

            trace!("Received data: {}", line);

            let line = line.trim_start_matches("<?xml version=\"1.0\" encoding=\"ISO-8859-1\" ?>");
            let root_node = util::parse_node(line)?;

            let expected_root = match self.mode {
                CodecMode::Client => "boinc_gui_rpc_reply",
                CodecMode::Server => "boinc_gui_rpc_request",
            };

            if root_node.name != expected_root {
                return Err(Error::DataParseError(format!(
                    "Invalid root: {}. Expected: {}",
                    root_node.name, expected_root
                )));
            }

            Ok(Some(root_node.children))
        } else {
            self.next_index = read_to;
            Ok(None)
        }
    }
}

impl Encoder<Vec<treexml::Element>> for BoincCodec {
    type Error = Error;

    fn encode(
        &mut self,
        item: Vec<treexml::Element>,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let mut out = treexml::Element::new(match self.mode {
            CodecMode::Client => "boinc_gui_rpc_request",
            CodecMode::Server => "boinc_gui_rpc_reply",
        });
        out.children = item;

        let data = format!("{}", out)
            .replace("<?xml version='1.0'?>", "")
            .replace(" />", "/>");

        trace!("Sending data: {}", data);
        dst.extend_from_slice(
            &ISO_8859_1
                .encode(&data, EncoderTrap::Strict)
                .expect("Our data should always be correct"),
        );
        dst.extend_from_slice(&[TERMCHAR]);
        Ok(())
    }
}

pub struct DaemonStream<Io> {
    conn: Framed<Io, BoincCodec>,
}

impl DaemonStream<TcpStream> {
    pub async fn connect(host: String, password: Option<String>) -> Result<Self, Error> {
        Self::authenticate(TcpStream::connect(host).await?, password).await
    }
}

impl<Io: AsyncRead + AsyncWrite + Unpin> DaemonStream<Io> {
    async fn authenticate(io: Io, password: Option<String>) -> Result<Self, Error> {
        let mut conn = BoincCodec::new(CodecMode::Client).framed(io);

        let mut out = Some(vec![treexml::Element::new("auth1")]);

        let mut nonce_sent = false;
        loop {
            if let Some(data) = out.take() {
                conn.send(data).await?;

                let data = conn
                    .try_next()
                    .await?
                    .ok_or_else(|| Error::DaemonError("EOF".into()))?;

                for node in data {
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

                            out = Some(vec![auth2_node]);
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
            } else {
                return Err(Error::DaemonError("Empty response".into()));
            }
        }
    }

    pub(crate) async fn query(
        &mut self,
        request_data: Vec<treexml::Element>,
    ) -> Result<Vec<treexml::Element>, Error> {
        self.conn.send(request_data).await?;
        let data = self
            .conn
            .try_next()
            .await?
            .ok_or_else(|| Error::DaemonError("EOF".into()))?;

        Ok(data)
    }
}
