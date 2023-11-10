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
abigen!(UniswapV3Factory, "./abi/uniswap_v3/UniswapV3Factory.json");
abigen!(UniswapV3Pool, "./abi/uniswap_v3/UniswapV3Pool.json");

pub async fn main(
    http_provider: Arc<Provider<Http>>,
    ws_provider: Arc<Provider<Ws>>,
    network: String,
    token_address_a: H160,
    token_address_b: H160,
    fee: u32, 
) -> Result<(), Box<dyn Error>> {

    dotenv::from_filename("./.env").ok();
    // Define the factory address for Ethereum, Polygon and BNB networks
    let factory_address_ethereum = env::var("UNISWAP_V3_FACTORY_ADDRESS_ETHEREUM")
        .expect("Expected FACTORY_ADDRESS_ETHEREUM in .env")
        .parse::<H160>()?;

    let factory_address_polygon = env::var("UNISWAP_V3_FACTORY_ADDRESS_POLYGON")
        .expect("Expected FACTORY_ADDRESS_POLYGON in .env")
        .parse::<H160>()?;

    let factory_address_bnb = env::var("UNISWAP_V3_FACTORY_ADDRESS_BNB")
        .expect("Expected FACTORY_ADDRESS_BNB in .env")
        .parse::<H160>()?;

    // Select the appropriate factory address based on the network
    let factory_address = match network.to_lowercase().as_str() {
        "ethereum" => factory_address_ethereum,
        "polygon" => factory_address_polygon,
        "bnb" => factory_address_bnb,
        _ => return Err("Unsupported network".into()),
    };

    // Create a contract instance using the ABI and address
    let factory = UniswapV3Factory::new(factory_address, Arc::clone(&http_provider));

    // Call the getPool function with the token addresses and fee
    let pool_address = factory.get_pool(token_address_a, token_address_b, fee).call().await?;

    println!("The pool address for the given token pair and fee tier is: {:?}", pool_address);

    // Create a contract instance using the ABI and address of the pool
    let pool = UniswapV3Pool::new(pool_address, Arc::clone(&http_provider));

    // Create ERC20 instances for both tokens
    let token_a = ERC20::new(token_address_a, http_provider.clone());
    let token_b = ERC20::new(token_address_b, http_provider.clone());

    // Fetch the token decimals
    let decimals_a = token_a.decimals().call().await? as u32;
    let decimals_b = token_b.decimals().call().await? as u32;

    // Calculate the absolute difference in decimals between token A and token B
    let decimal_diff = (decimals_a as i32 - decimals_b as i32).abs();

    compute_and_print_prices(token_address_a, token_address_b, decimal_diff, pool.clone()).await?;

        
    // Set up a filter for the Swap event that matches the event signature
    let filter = Filter::new()
        .event("Swap(address,address,int256,int256,uint160,uint128,int24)")
        .address(vec![pool_address]);

    // Subscribe to the Swap events
    let mut stream: SubscriptionStream<'_, Ws, Log> = ws_provider.subscribe_logs(&filter).await?;
    println!("\nListening for Swap events...");

    // Process events as they come in
    while let Some(log) = stream.next().await {
        compute_and_print_prices(token_address_a, token_address_b, decimal_diff, pool.clone()).await?;
    }
    

    Ok(())
}


async fn compute_and_print_prices(
    token_address_a: H160,
    token_address_b: H160,
    decimal_diff: i32,
    pool: UniswapV3Pool<Provider<Http>>
) -> Result<(), Box<dyn Error>> {
    
    // Get the current slot0 information, which includes the current price
    let slot0 = pool.slot_0().call().await?;

    // Destructure the tuple to get the values
    let (sqrt_price_x96, tick, observation_index, observation_cardinality, observation_cardinality_next, fee_protocol, unlocked) = slot0;

    // println!("Current price (sqrtPriceX96): {:?}", sqrt_price_x96);

    // Calculate price of Token A in terms of Token B (Price A/B)
    // First, convert sqrt_price_x96 to a floating-point number for the calculation
    let price_ratio: f64 = (sqrt_price_x96.low_u128() as f64).powi(2);

    let price_a_per_b: f64;
    let price_b_per_a: f64;

    // Determine which token is token0 and token1 based on address
    if token_address_a < token_address_b {
        // token A is token0 and token B is token1
        price_a_per_b = price_ratio / ((1u128 << 96) as f64).powi(2) * 10f64.powi(decimal_diff);
        price_b_per_a = 1.0 / price_a_per_b;
    } else {
        // token B is token0 and token A is token1
        price_b_per_a = price_ratio / ((1u128 << 96) as f64).powi(2) * 10f64.powi(decimal_diff);
        price_a_per_b = 1.0 / price_b_per_a;
    }

    // Print the prices, ensuring that they are labeled according to the token addresses
    let now = Local::now();
    println!("\n[{time}] Price of token A in terms of B: {price_a_b:.18}", time=now.format("%H:%M:%S"), price_a_b=price_a_per_b);
    println!("[{time}] Price of token B in terms of A: {price_b_a:.18}", time=now.format("%H:%M:%S"), price_b_a=price_b_per_a);

    Ok(())
}