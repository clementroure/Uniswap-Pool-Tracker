mod config;
mod uniswap {
    pub mod uniswap_v2;
    pub mod uniswap_v3;
}

use ethers::providers::{Http, Provider, Ws};
use std::{error::Error, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = config::init_args()?;

    let http_provider = Arc::new(Provider::<Http>::try_from(config.rpc_url.as_str())?);
    let ws_provider = Arc::new(Provider::new(Ws::connect(config.ws_endpoint).await?));

    // Handle the Result returned by uniswap_v2 and uniswap_v3 functions
    match config.uniswap_version.as_str() {
        "uniswapv2" => {
            uniswap::uniswap_v2::main(http_provider, ws_provider, config.token_address_a, config.token_address_b).await?;
        },
        "uniswapv3" => {
            uniswap::uniswap_v3::main(http_provider, ws_provider, config.network, config.token_address_a, config.token_address_b, config.fee).await?;
        },
        _ => unreachable!(), // we've already validated the version in config::init_args
    }

    Ok(())
}