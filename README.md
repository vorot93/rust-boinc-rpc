# boinc-rpc-rs

[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![GitHub Actions workflow status](https://github.com/vorot93/boinc-rpc-rs/workflows/Continuous%20integration/badge.svg)](https://github.com/vorot93/boinc-rpc-rs/actions)

Rust client for BOINC.

## Usage example
```
#[tokio::main]
async fn main() {
    let mut client = boinc_rpc::Client::connect("127.0.0.1:31416", Some("my-pass-in-gui_rpc_auth.cfg".into())).await.unwrap();

    println!("{:?}\n", client.get_messages(0).await.unwrap());
    println!("{:?}\n", client.get_projects().await.unwrap());
    println!("{:?}\n", client.get_account_manager_info().await.unwrap());
    println!("{:?}\n", client.exchange_versions(&rpc::models::VersionInfo::default()).await.unwrap());
    println!("{:?}\n", client.get_results(false).await.unwrap());
}
```
