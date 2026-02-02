use bdk_bitcoind_client::bitreq::Auth;

// Usage:
// $ export RPC_COOKIE="/path/to/.bitcoin/.cookie"
// $ cargo run --example bitreq

const URL: &str = "http://127.0.0.1:18443";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize RPC client
    let cookie_file = std::env::var("RPC_COOKIE").expect("must set RPC_COOKIE");
    let client = bdk_bitcoind_client::bitreq::Client::new(
        URL,
        std::time::Duration::from_secs(15),
        Auth::CookieFile(cookie_file.into()),
    )?;

    // Get blockhain info
    println!("{:#?}", client.get_blockchain_info());

    Ok(())
}
