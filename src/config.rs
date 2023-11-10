use ethers::types::H160;
use std::env;
use std::str::FromStr;
use std::error::Error;

pub struct Config {
    pub network: String,
    pub uniswap_version: String,
    pub token_address_a: H160,
    pub token_address_b: H160,
    pub rpc_url: String,
    pub ws_endpoint: String,
    pub fee: u32, // Optional, because it's only used for Uniswap V3
}

pub fn init_args() -> Result<Config, Box<dyn Error>> {
    dotenv::from_filename("./.env").ok();

    // Check if the command has the right number of inputs
    let args: Vec<String> = env::args().collect();
    if (args[2] == "uniswapv3" && args.len() != 6) || (args[2] == "uniswapv2" && args.len() != 5) {
        eprintln!("Usage for Uniswap V2: cargo run <network> <uniswap_version> <token_address_a> <token_address_b>");
        eprintln!("Usage for Uniswap V3: cargo run <network> <uniswap_version> <token_address_a> <token_address_b> <fee>");
        eprintln!("Fee for Uniswap V3 must be one of: 500, 3000, 10000");
        return Err("Invalid number of arguments".into());
    }

    let network = args[1].clone();
    let uniswap_version = args[2].clone();
    let token_address_a_str = args[3].clone();
    let token_address_b_str = args[4].clone();

    if uniswap_version != "uniswapv2" && uniswap_version != "uniswapv3" {
        return Err("Unsupported DEX".into());
    }

    if uniswap_version == "uniswapv2" && network != "ethereum" {
        return Err("Uniswap V2 only supports the Ethereum network".into());
    }
    
    if uniswap_version == "uniswapv3" && !["ethereum", "polygon", "bnb"].contains(&network.as_str()) {
        return Err("Uniswap V3 supports Ethereum, Polygon, and BNB networks".into());
    }

    let token_address_a = H160::from_str(&token_address_a_str)
        .map_err(|_| "Invalid token address format for token A")?;
    let token_address_b = H160::from_str(&token_address_b_str)
        .map_err(|_| "Invalid token address format for token B")?;

    // Additional variable to store the fee, which is only relevant for Uniswap V3
    let fee: u32 = if uniswap_version == "uniswapv3" {
        let fee_str = args[5].clone();
        match fee_str.as_str() {
            "500" | "3000" | "10000" => fee_str.parse::<u32>()?,
            _ => return Err("Invalid fee specified for Uniswap V3".into()),
        }
    } else {
        0 // setting a default fee to 0 for Uniswap V2, we dont use it
    };    

    // API keys are fetched from the environment
    let eth_api_key = env::var("ETH_ALCHEMY_API_KEY").expect("ETH_ALCHEMY_API_KEY not set");
    let pol_api_key = env::var("POL_ALCHEMY_API_KEY").expect("POL_ALCHEMY_API_KEY not set");
    let bnb_api_key = env::var("BNB_QUICKNODE_API_KEY").expect("BNB_QUICKNODE_API_KEY not set");

    // URLs are constructed by concatenating the base URL with the API keys
    let rpc_url = match network.as_str() {
        "ethereum" => format!("https://eth-mainnet.g.alchemy.com/v2/{}", eth_api_key),
        "polygon" => format!("https://polygon-mainnet.g.alchemy.com/v2/{}", pol_api_key),
        "bnb" => format!("https://blue-practical-patron.bsc.quiknode.pro/{}", bnb_api_key),
        _ => return Err("Unsupported network specified".into()),
    };

    let ws_endpoint = match network.as_str() {
        "ethereum" => format!("wss://eth-mainnet.g.alchemy.com/v2/{}", eth_api_key),
        "polygon" => format!("wss://polygon-mainnet.g.alchemy.com/v2/{}", pol_api_key),
        "bnb" => format!("wss://blue-practical-patron.bsc.quiknode.pro/{}", bnb_api_key),
        _ => return Err("Unsupported network specified".into()),
    };

    Ok(Config {
        network,
        uniswap_version,
        token_address_a,
        token_address_b,
        rpc_url,
        ws_endpoint,
        fee,
    })
}
