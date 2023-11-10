use std::env;
use std::error::Error;
use std::sync::Arc;
use chrono::Local;
use ethers::providers::Provider;
use ethers::providers::Http;
use ethers::prelude::abigen;
use ethers::providers::Ws;
use ethers::types::H160;
use ethers::prelude::*; 
use futures::stream::StreamExt;

abigen!(ERC20, "./abi/ERC20.json");
abigen!(UniswapV2Pair, "./abi/uniswap_v2/UniswapV2Pair.json");
abigen!(UniswapV2Factory, "./abi/uniswap_v2/UniswapV2factory.json");

pub async fn main(
    http_provider: Arc<Provider<Http>>,
    ws_provider: Arc<Provider<Ws>>,
    token_address_a: H160,
    token_address_b: H160,
) -> Result<(), Box<dyn Error>> {

    dotenv::from_filename("./.env").ok();
    // Define the factory address for Ethereum
    let factory_address_ethereum = env::var("UNISWAP_V2_FACTORY_ADDRESS_ETHEREUM")
        .expect("Expected FACTORY_ADDRESS_ETHEREUM in .env")
        .parse::<H160>()?;

    // Create instances of the contract bindings
    let uniswap_v2_factory = UniswapV2Factory::new(factory_address_ethereum, http_provider.clone());

    // Fetch the pair address using the provided token addresses
    let pair_address = uniswap_v2_factory.get_pair(token_address_a, token_address_b).call().await?;

    println!("Pair Address: {:?}", pair_address);

    // Create an instance of the Uniswap V2 Pair contract
    let uniswap_v2_pair = UniswapV2Pair::new(pair_address, http_provider.clone()).clone();

    // Create ERC20 instances for both tokens
    let token_a = ERC20::new(token_address_a, http_provider.clone());
    let token_b = ERC20::new(token_address_b, http_provider.clone());

    // Fetch the token decimals
    let decimals_a = token_a.decimals().call().await? as u32;
    let decimals_b = token_b.decimals().call().await? as u32;

    compute_and_print_prices(token_address_a, token_address_b, decimals_a, decimals_b, uniswap_v2_pair.clone()).await?;
  
    let filter = Filter::new()
        // .event("Sync(uint112,uint112)") // this event catch both swaps and liquidity providing
        .event("Swap(address,uint256,uint256,uint256,uint256,address)")
        .address(vec![pair_address]);

    let mut stream: SubscriptionStream<'_, Ws, Log> = ws_provider.subscribe_logs(&filter).await?;
    println!("\nListening for Swap events...");

    // Process events as they come in
    while let Some(log) = stream.next().await {
        // println!("Log: {:?}", log);
        compute_and_print_prices(token_address_a, token_address_b, decimals_a, decimals_b, uniswap_v2_pair.clone()).await?;
    }

    Ok(())
}

async fn compute_and_print_prices(
    token_address_a: H160,
    token_address_b: H160,
    decimals_a: u32,
    decimals_b: u32,
    uniswap_v2_pair: UniswapV2Pair<Provider<Http>>
) -> Result<(), Box<dyn Error>> {
    let reserves = uniswap_v2_pair.get_reserves().call().await?;
    let (reserve0, reserve1, _) = reserves;

    // Ensure the reserves are ordered to match the token addresses
    // In Uniswap, token0 is the token with the smaller address
    let (reserve_a, reserve_b) = if token_address_a < token_address_b {
        (reserve0, reserve1)
    } else {
        (reserve1, reserve0)
    };

    // Normalize the reserves to a common basis for comparison
    let common_decimals = 18;
    let reserve_a_adjusted = reserve_a * 10u128.pow(common_decimals - decimals_a);
    let reserve_b_adjusted = reserve_b * 10u128.pow(common_decimals - decimals_b);

    // Calculate the price of 1 unit of token A in terms of token B
    let price_of_a_in_terms_of_b = reserve_b_adjusted as f64 / reserve_a_adjusted as f64;
    // Calculate the price of 1 unit of token B in terms of token A
    let price_of_b_in_terms_of_a = reserve_a_adjusted as f64 / reserve_b_adjusted as f64;

    let now = Local::now();
    println!("\n[{time}] Price of token A in terms of B: {price_a_b:.18}", time=now.format("%H:%M:%S"), price_a_b=price_of_a_in_terms_of_b);
    println!("[{time}] Price of token B in terms of A: {price_b_a:.18}", time=now.format("%H:%M:%S"), price_b_a=price_of_b_in_terms_of_a);

    Ok(())
}