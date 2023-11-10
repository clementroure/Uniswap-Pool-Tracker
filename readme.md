
# DeFi Price Streaming

This program streams price data from Uniswap V2 or V3, allowing users to specify token addresses for Uniswap V2 or chain (Ethereum, Polygon, BNB), token addresses, and pool fee for Uniswap V3.

## Installation and Running the Program

### Prerequisites
- Rust programming language
- Cargo (Rust's package manager)

### Installation
1. Clone the repository:
   ```
   git clone https://github.com/clementroure/price-streaming
   ```
2. Navigate to the project directory:
   ```
   cd price_streaming
   ```
3. Install dependencies as specified in cargo.toml:
   ```
   cargo build
   ```
   
### Running the Program
Run the program using Cargo:
```
cargo run [network] [uniswap_version] [token_address_a] [token_address_b] [optional: fee]
```

## Usage

### Uniswap V2
To stream prices from Uniswap V2 on Ethereum, specify two token addresses:
```
cargo run ethereum uniswapv2 <token_address_a> <token_address_b>
```

#### Example for Uniswap V2
Ethereum Network (WETH and USDT):
```
cargo run ethereum uniswapv2 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 0xdAC17F958D2ee523a2206206994597C13D831ec7
```

### Uniswap V3
For Uniswap V3, specify the chain, two token addresses, and the fee of the pool:
```
cargo run <network> uniswapv3 <token_address_a> <token_address_b> <fee>
```
Supported networks are Ethereum, Polygon, and BNB. Valid fee values are 500, 3000, or 10000.

#### Examples for Uniswap V3
- Ethereum Network:
  ```
  cargo run ethereum uniswapv3 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 0xdAC17F958D2ee523a2206206994597C13D831ec7 3000
  ```
- Polygon Network:
  ```
  cargo run polygon uniswapv3 0xc2132D05D31c914a87C6611C10748AEb04B58e8F 0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619 3000
  ```
- BNB Network:
  ```
  cargo run bnb uniswapv3 0x55d398326f99059fF775485246999027B3197955 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c 3000
  ```

## Code Explanation

### Modules
- `config`: Contains `Config` struct and `init_args()` function for initializing program configurations.
- `uniswap`: Includes `uniswap_v2` and `uniswap_v3` modules for handling different Uniswap versions.

### Main Components
- `Config` struct: Holds the configuration parameters required for the program to run.
- `init_args()`: Parses and validates command-line arguments, returning a `Config` object.

### Error Handling
The program checks for valid input arguments and network support for each Uniswap version. Errors are thrown for invalid inputs.

### Network Requests
- `rpc_url` and `ws_endpoint`: Constructed based on the provided network argument.
- `ethers::providers::{Http, Provider, Ws}`: Used to interact with Ethereum and other blockchain networks.

### Asynchronous Execution
- The `main` function is an asynchronous entry point using `#[tokio::main]`.
- It handles Uniswap V2 and V3 functionality based on user inputs.