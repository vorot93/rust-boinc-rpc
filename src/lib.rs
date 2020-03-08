//! Rust client for BOINC RPC protocol.
//!
//! # Example
//!
//! ```rust,no_run
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let transport = boinc_rpc::Transport::new("127.0.0.1:31416", Some("my-pass-in-gui_rpc_auth.cfg"));
//! let mut client = boinc_rpc::Client::new(transport);
//!
//! println!("{:?}\n", client.get_messages(0).await.unwrap());
//! println!("{:?}\n", client.get_projects().await.unwrap());
//! println!("{:?}\n", client.get_account_manager_info().await.unwrap());
//! println!("{:?}\n", client.exchange_versions(&boinc_rpc::models::VersionInfo::default()).await.unwrap());
//! println!("{:?}\n", client.get_results(false).await.unwrap());
//! # })
//! ```

#![allow(clippy::type_complexity)]

pub mod errors;
pub mod models;
pub mod rpc;
pub mod util;

use crate::{errors::*, rpc::*};
use std::{
    fmt::Display,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::{net::TcpStream, sync::Mutex};
use tower::ServiceExt;

pub fn verify_rpc_reply_contents(data: &[treexml::Element]) -> Result<bool, Error> {
    let mut success = false;
    for node in data {
        match &*node.name {
            "success" => success = true,
            "status" => {
                return Err(Error::StatusError(
                    util::eval_node_contents(&node).unwrap_or(9999),
                ));
            }
            "unauthorized" => {
                return Err(Error::AuthError(String::new()));
            }
            "error" => {
                let error_msg = node
                    .text
                    .clone()
                    .ok_or_else(|| Error::DaemonError("Unknown error".into()))?;

                return match &*error_msg {
                    "unauthorized" => Err(Error::AuthError(error_msg)),
                    "Missing authenticator" => Err(Error::AuthError(error_msg)),
                    "Missing URL" => Err(Error::InvalidURLError(error_msg)),
                    "Already attached to project" => Err(Error::AlreadyAttachedError(error_msg)),
                    _ => Err(Error::DataParseError(error_msg)),
                };
            }
            _ => {}
        }
    }
    Ok(success)
}

impl<'a> From<&'a treexml::Element> for models::Message {
    fn from(node: &treexml::Element) -> models::Message {
        let mut e = models::Message::default();
        for n in &node.children {
            match &*n.name {
                "body" => {
                    e.body = util::trimmed_optional(&n.cdata);
                }
                "project" => {
                    e.project_name = util::trimmed_optional(&n.text);
                }
                "pri" => {
                    e.priority = util::eval_node_contents(&n);
                }
                "seqno" => {
                    e.msg_number = util::eval_node_contents(&n);
                }
                "time" => {
                    e.timestamp = util::eval_node_contents(&n);
                }
                _ => {}
            }
        }

        e
    }
}

impl<'a> From<&'a treexml::Element> for models::ProjectInfo {
    fn from(node: &treexml::Element) -> models::ProjectInfo {
        let mut e = models::ProjectInfo::default();
        for n in &node.children {
            match &*n.name {
                "name" => {
                    e.name = util::trimmed_optional(&util::any_text(n));
                }
                "summary" => {
                    e.summary = util::trimmed_optional(&util::any_text(n));
                }
                "url" => {
                    e.url = util::trimmed_optional(&util::any_text(n));
                }
                "general_area" => {
                    e.general_area = util::trimmed_optional(&util::any_text(n));
                }
                "specific_area" => {
                    e.specific_area = util::trimmed_optional(&util::any_text(n));
                }
                "description" => {
                    e.description = util::trimmed_optional(&util::any_text(n));
                }
                "home" => {
                    e.home = util::trimmed_optional(&util::any_text(n));
                }
                "platfroms" => {
                    let mut platforms = Vec::new();
                    for platform_node in &n.children {
                        if platform_node.name == "platform" {
                            if let Some(v) = &platform_node.text {
                                platforms.push(v.clone());
                            }
                        }
                    }
                    e.platforms = Some(platforms);
                }
                "image" => {
                    e.image = util::trimmed_optional(&util::any_text(n));
                }
                _ => {}
            }
        }

        e
    }
}

impl<'a> From<&'a treexml::Element> for models::AccountManagerInfo {
    fn from(node: &treexml::Element) -> models::AccountManagerInfo {
        let mut e = models::AccountManagerInfo::default();
        for n in &node.children {
            match &*n.name {
                "acct_mgr_url" => e.url = util::trimmed_optional(&util::any_text(n)),
                "acct_mgr_name" => e.name = util::trimmed_optional(&util::any_text(n)),
                "have_credentials" => {
                    e.have_credentials = Some(true);
                }
                "cookie_required" => {
                    e.cookie_required = Some(true);
                }
                "cookie_failure_url" => {
                    e.cookie_failure_url = util::trimmed_optional(&util::any_text(n))
                }
                _ => {}
            }
        }
        e
    }
}

impl<'a> From<&'a treexml::Element> for models::VersionInfo {
    fn from(node: &treexml::Element) -> models::VersionInfo {
        let mut e = models::VersionInfo::default();
        for n in &node.children {
            match &*n.name {
                "major" => e.major = util::eval_node_contents(&n),
                "minor" => e.minor = util::eval_node_contents(&n),
                "release" => e.release = util::eval_node_contents(&n),
                _ => {}
            }
        }
        e
    }
}

impl<'a> From<&'a treexml::Element> for models::TaskResult {
    fn from(node: &treexml::Element) -> models::TaskResult {
        let mut e = models::TaskResult::default();
        for n in &node.children {
            match &*n.name {
                "name" => {
                    e.name = util::trimmed_optional(&n.text);
                }
                "wu_name" => {
                    e.wu_name = util::trimmed_optional(&n.text);
                }
                "platform" => {
                    e.platform = util::trimmed_optional(&n.text);
                }
                "version_num" => {
                    e.version_num = util::eval_node_contents(&n);
                }
                "plan_class" => {
                    e.plan_class = util::trimmed_optional(&n.text);
                }
                "project_url" => {
                    e.project_url = util::trimmed_optional(&n.text);
                }
                "final_cpu_time" => {
                    e.final_cpu_time = util::eval_node_contents(&n);
                }
                "final_elapsed_time" => {
                    e.final_elapsed_time = util::eval_node_contents(&n);
                }
                "exit_status" => {
                    e.exit_status = util::eval_node_contents(&n);
                }
                "state" => {
                    e.state = util::eval_node_contents(&n);
                }
                "report_deadline" => {
                    e.report_deadline = util::eval_node_contents(&n);
                }
                "received_time" => {
                    e.received_time = util::eval_node_contents(&n);
                }
                "estimated_cpu_time_remaining" => {
                    e.estimated_cpu_time_remaining = util::eval_node_contents(&n);
                }
                "completed_time" => {
                    e.completed_time = util::eval_node_contents(&n);
                }
                "active_task" => {
                    e.active_task = Some(models::ActiveTask::from(n));
                }
                _ => {}
            }
        }
        e
    }
}

impl<'a> From<&'a treexml::Element> for models::HostInfo {
    fn from(node: &treexml::Element) -> models::HostInfo {
        let mut e = models::HostInfo::default();
        for n in &node.children {
            match &*n.name {
                "p_fpops" => e.p_fpops = util::eval_node_contents(&n),
                "p_iops" => e.p_iops = util::eval_node_contents(&n),
                "p_membw" => e.p_membw = util::eval_node_contents(&n),
                "p_calculated" => e.p_calculated = util::eval_node_contents(&n),
                "p_vm_extensions_disabled" => {
                    e.p_vm_extensions_disabled = util::eval_node_contents(&n)
                }
                "host_cpid" => e.host_cpid = n.text.clone(),
                "product_name" => e.product_name = n.text.clone(),
                "mac_address" => e.mac_address = n.text.clone(),
                "domain_name" => e.domain_name = n.text.clone(),
                "ip_addr" => e.ip_addr = n.text.clone(),
                "p_vendor" => e.p_vendor = n.text.clone(),
                "p_model" => e.p_model = n.text.clone(),
                "os_name" => e.os_name = n.text.clone(),
                "os_version" => e.os_version = n.text.clone(),
                "virtualbox_version" => e.virtualbox_version = n.text.clone(),
                "p_features" => e.p_features = n.text.clone(),
                "timezone" => e.tz_shift = util::eval_node_contents(&n),
                "p_ncpus" => e.p_ncpus = util::eval_node_contents(&n),
                "m_nbytes" => e.m_nbytes = util::eval_node_contents(&n),
                "m_cache" => e.m_cache = util::eval_node_contents(&n),
                "m_swap" => e.m_swap = util::eval_node_contents(&n),
                "d_total" => e.d_total = util::eval_node_contents(&n),
                "d_free" => e.d_free = util::eval_node_contents(&n),
                _ => {}
            }
        }
        e
    }
}

type DaemonStreamFuture =
    Pin<Box<dyn Future<Output = Result<DaemonStream<TcpStream>, Error>> + Send + Sync + 'static>>;

enum ConnState {
    Connecting(DaemonStreamFuture),
    Ready(DaemonStream<TcpStream>),
    Error(Error),
}

pub struct Transport {
    state: Arc<Mutex<Option<ConnState>>>,
}

impl Transport {
    pub fn new<A: Display, P: Display>(addr: A, password: Option<P>) -> Self {
        let addr = addr.to_string();
        let password = password.map(|p| p.to_string());
        Self {
            state: Arc::new(Mutex::new(Some(ConnState::Connecting(Box::pin(
                DaemonStream::connect(addr, password),
            ))))),
        }
    }
}

impl tower::Service<Vec<treexml::Element>> for Transport {
    type Response = Vec<treexml::Element>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut g = match self.state.try_lock() {
            Ok(g) => g,
            Err(_) => return Poll::Pending,
        };

        let (state, out) = match g.take().unwrap() {
            ConnState::Connecting(mut future) => {
                let res = future.as_mut().poll(cx);
                match res {
                    Poll::Pending => (Some(ConnState::Connecting(future)), Poll::Pending),
                    Poll::Ready(Ok(conn)) => (Some(ConnState::Ready(conn)), Poll::Ready(Ok(()))),
                    Poll::Ready(Err(e)) => (None, Poll::Ready(Err(e))),
                }
            }
            ConnState::Ready(conn) => (Some(ConnState::Ready(conn)), Poll::Ready(Ok(()))),
            ConnState::Error(error) => (
                Some(ConnState::Error(error.clone())),
                Poll::Ready(Err(error)),
            ),
        };

        *g = state;
        out
    }

    fn call(&mut self, req: Vec<treexml::Element>) -> Self::Future {
        let state = self.state.clone();
        Box::pin(async move {
            let mut state = state.lock().await;

            let mut conn = match state.take() {
                Some(ConnState::Ready(conn)) => conn,
                _ => unreachable!(),
            };

            let res = conn.query(req).await;

            if let Err(e) = &res {
                *state = Some(ConnState::Error(e.clone()));
            }

            res
        })
    }
}

pub struct Client<S> {
    transport: S,
}

impl<S> Client<S>
where
    S: tower::Service<Vec<treexml::Element>, Response = Vec<treexml::Element>, Error = Error>,
{
    pub fn new(transport: S) -> Self {
        Self { transport }
    }

    async fn get_object<T: for<'a> From<&'a treexml::Element>>(
        &mut self,
        req_data: Vec<treexml::Element>,
        object_tag: &str,
    ) -> Result<T, Error> {
        self.transport.ready().await?;
        let data = self.transport.call(req_data).await?;
        verify_rpc_reply_contents(&data)?;
        for child in &data {
            if child.name == object_tag {
                return Ok(T::from(child));
            }
        }
        Err(Error::DataParseError("Object not found.".to_string()))
    }

    async fn get_object_by_req_tag<T: for<'a> From<&'a treexml::Element>>(
        &mut self,
        req_tag: &str,
        object_tag: &str,
    ) -> Result<T, Error> {
        self.get_object(vec![treexml::Element::new(req_tag)], object_tag)
            .await
    }

    async fn get_vec<T: for<'a> From<&'a treexml::Element>>(
        &mut self,
        req_data: Vec<treexml::Element>,
        vec_tag: &str,
        object_tag: &str,
    ) -> Result<Vec<T>, Error> {
        let mut v = Vec::new();
        {
            self.transport.ready().await?;
            let data = self.transport.call(req_data).await?;
            verify_rpc_reply_contents(&data)?;
            let mut success = false;
            for child in data {
                if child.name == vec_tag {
                    success = true;
                    for vec_child in &child.children {
                        if vec_child.name == object_tag {
                            v.push(T::from(vec_child));
                        }
                    }
                }
            }
            if !success {
                return Err(Error::DataParseError("Objects not found.".to_string()));
            }
        }
        Ok(v)
    }

    async fn get_vec_by_req_tag<T: for<'a> From<&'a treexml::Element>>(
        &mut self,
        req_tag: &str,
        vec_tag: &str,
        object_tag: &str,
    ) -> Result<Vec<T>, Error> {
        self.get_vec(vec![treexml::Element::new(req_tag)], vec_tag, object_tag)
            .await
    }

    pub async fn get_messages(&mut self, seqno: i64) -> Result<Vec<models::Message>, Error> {
        self.get_vec(
            vec![{
                let mut node = treexml::Element::new("get_messages");
                node.text = Some(format!("{}", seqno));
                node
            }],
            "msgs",
            "msg",
        )
        .await
    }

    pub async fn get_projects(&mut self) -> Result<Vec<models::ProjectInfo>, Error> {
        self.get_vec_by_req_tag("get_all_projects_list", "projects", "project")
            .await
    }

    pub async fn get_account_manager_info(&mut self) -> Result<models::AccountManagerInfo, Error> {
        self.get_object_by_req_tag("acct_mgr_info", "acct_mgr_info")
            .await
    }

    pub async fn get_account_manager_rpc_status(&mut self) -> Result<i32, Error> {
        self.transport.ready().await?;
        let data = self
            .transport
            .call(vec![treexml::Element::new("acct_mgr_rpc_poll")])
            .await?;
        verify_rpc_reply_contents(&data)?;

        let mut v: Option<i32> = None;
        for child in &data {
            if &*child.name == "acct_mgr_rpc_reply" {
                for c in &child.children {
                    if &*c.name == "error_num" {
                        v = util::eval_node_contents(&c);
                    }
                }
            }
        }
        v.ok_or_else(|| Error::DataParseError("acct_mgr_rpc_reply node not found".into()))
    }

    pub async fn connect_to_account_manager(
        &mut self,
        url: &str,
        name: &str,
        password: &str,
    ) -> Result<bool, Error> {
        let mut req_node = treexml::Element::new("acct_mgr_rpc");
        req_node.children = vec![
            {
                let mut node = treexml::Element::new("url");
                node.text = Some(url.into());
                node
            },
            {
                let mut node = treexml::Element::new("name");
                node.text = Some(name.into());
                node
            },
            {
                let mut node = treexml::Element::new("password");
                node.text = Some(password.into());
                node
            },
        ];
        self.transport.ready().await?;
        let root_node = self.transport.call(vec![req_node]).await?;
        Ok(verify_rpc_reply_contents(&root_node)?)
    }

    pub async fn exchange_versions(
        &mut self,
        info: &models::VersionInfo,
    ) -> Result<models::VersionInfo, Error> {
        let mut content_node = treexml::Element::new("exchange_versions");
        {
            let mut node = treexml::Element::new("major");
            node.text = info.minor.map(|v| format!("{}", v));
            content_node.children.push(node);
        }
        {
            let mut node = treexml::Element::new("minor");
            node.text = info.major.map(|v| format!("{}", v));
            content_node.children.push(node);
        }
        {
            let mut node = treexml::Element::new("release");
            node.text = info.release.map(|v| format!("{}", v));
            content_node.children.push(node);
        }
        self.get_object(vec![content_node], "server_version").await
    }

    pub async fn get_results(
        &mut self,
        active_only: bool,
    ) -> Result<Vec<models::TaskResult>, Error> {
        self.get_vec(
            vec![{
                let mut node = treexml::Element::new("get_results");
                if active_only {
                    let mut ao_node = treexml::Element::new("active_only");
                    ao_node.text = Some("1".into());
                    node.children.push(ao_node);
                }
                node
            }],
            "results",
            "result",
        )
        .await
    }

    pub async fn set_mode(
        &mut self,
        c: models::Component,
        m: models::RunMode,
        duration: f64,
    ) -> Result<(), Error> {
        self.transport.ready().await?;
        let rsp_root = self
            .transport
            .call(vec![{
                let comp_desc = match c {
                    models::Component::CPU => "run",
                    models::Component::GPU => "gpu",
                    models::Component::Network => "network",
                }
                .to_string();
                let mode_desc = match m {
                    models::RunMode::Always => "always",
                    models::RunMode::Auto => "auto",
                    models::RunMode::Never => "never",
                    models::RunMode::Restore => "restore",
                }
                .to_string();

                let mut node = treexml::Element::new(format!("set_{}_mode", &comp_desc));
                let mut dur_node = treexml::Element::new("duration");
                dur_node.text = Some(format!("{}", duration));
                node.children.push(dur_node);
                node.children.push(treexml::Element::new(mode_desc));
                node
            }])
            .await?;
        verify_rpc_reply_contents(&rsp_root)?;
        Ok(())
    }

    pub async fn get_host_info(&mut self) -> Result<models::HostInfo, Error> {
        self.get_object_by_req_tag("get_host_info", "host_info")
            .await
    }

    pub async fn set_language(&mut self, v: &str) -> Result<(), Error> {
        self.transport.ready().await?;
        verify_rpc_reply_contents(
            &self
                .transport
                .call(vec![{
                    let mut node = treexml::Element::new("set_language");
                    let mut language_node = treexml::Element::new("language");
                    language_node.text = Some(v.into());
                    node.children.push(language_node);
                    node
                }])
                .await?,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::errors::Error;

    #[test]
    fn verify_rpc_reply_contents() {
        let mut fixture = treexml::Element::new("error");
        fixture.text = Some("Missing authenticator".into());
        let fixture = vec![fixture];
        assert_eq!(
            super::verify_rpc_reply_contents(&fixture).err().unwrap(),
            Error::AuthError("Missing authenticator".to_string())
        );
    }
}
