// async client example

use corepc_types::v29;
use jsonrpc::base64::Engine;
use jsonrpc::base64::engine::general_purpose::STANDARD as BASE64;
use jsonrpc::bitreq;

const URL: &str = "http://127.0.0.1:18443";
const RPC_COOKIE_PATH: &str = ".bitcoin/regtest/.cookie";

#[tokio::main]
async fn main() -> Result<(), Box<dyn core::error::Error>> {
    // Create cookie authentication
    let cookie_file = std::env::var("RPC_COOKIE").unwrap_or(RPC_COOKIE_PATH.to_string());
    let cookie = std::fs::read_to_string(cookie_file)?;
    let auth_header = format!("Basic {}", BASE64.encode(cookie.as_bytes()));

    // The RPC method to call
    let rpc = bdk_bitcoind_client::Rpc::GetBlockchainInfo;
    let method = rpc.to_string();

    // Create RPC client
    let client = bdk_bitcoind_client::Client::new();

    // The `send_fn` takes the request as a JSON value, sends it asynchronously,
    // and parses the response as a `jsonrpc::Response`.
    let send_fn = async |request: &jsonrpc::Request| {
        bitreq::post(URL)
            .with_header("Authorization", auth_header.clone())
            .with_json(request)?
            .send_async()
            .await
            .unwrap()
            .json::<jsonrpc::Response>()
    };

    // Execute the RPC
    let get_blockchain_info = client
        .call_async::<v29::GetBlockchainInfo, bitreq::Error>(&method, &[], send_fn)
        .await?
        .into_model()?;

    println!("{:#?}", get_blockchain_info);

    Ok(())
}
