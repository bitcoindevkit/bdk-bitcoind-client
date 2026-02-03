use bdk_bitcoind_client::Rpc;
use corepc_types::v29;
use jsonrpc::base64::engine::general_purpose::STANDARD as BASE64;
use jsonrpc::base64::Engine;

// Usage:
// $ export RPC_COOKIE="/path/to/.bitcoin/.cookie"
// $ cargo run --example async

const URL: &str = "http://127.0.0.1:18443";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cookie_file = std::env::var("RPC_COOKIE").expect("must set RPC_COOKIE");
    let cookie = std::fs::read_to_string(cookie_file)?;

    // Create RPC client
    let client = bdk_bitcoind_client::Client::new();

    // Implement async send request function
    let auth_header = format!("Basic {}", BASE64.encode(cookie.as_bytes()));
    let send_fn = move |request_value: serde_json::Value| {
        let auth_header = auth_header.clone();
        async move {
            bitreq::post(URL)
                .with_header("Authorization", auth_header)
                .with_json(&request_value)?
                .send_async()
                .await?
                .json::<jsonrpc::Response>()
        }
    };

    // Execute the RPC
    let blockchain_info = client
        .call_async::<v29::GetBlockchainInfo, _, _, _>(Rpc::GetBlockchainInfo, &[], send_fn)
        .await?
        .into_model()?;

    println!("{:#?}", blockchain_info);

    Ok(())
}
