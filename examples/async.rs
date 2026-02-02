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

    // Implement send request function
    use jsonrpc::{Request, Response};
    let send_fn = async |request: Request| -> Result<Response, bitreq::Error> {
        bitreq::post(URL)
            .with_header(
                "Authorization",
                format!("Basic {}", BASE64.encode(cookie.as_bytes())),
            )
            .with_json(&request)?
            .send_async()
            .await?
            .json::<jsonrpc::Response>()
    };

    // Execute the RPC
    let blockchain_info = client
        .call_async::<v29::GetBlockchainInfo, _>(Rpc::GetBlockchainInfo, &[], send_fn)
        .await?
        .into_model()?;

    println!("{:#?}", blockchain_info);

    Ok(())
}
