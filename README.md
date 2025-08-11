# Argus - Cross-Chain Arbitrage Monitor

Track WETH/USDC price differences between Uniswap V4 (Ethereum) and Aerodrome (Base) to spot arbitrage opportunities in real-time.

<p align="center">
  <a href="https://github.com/tokio-rs/tokio">
    <img src="https://img.shields.io/badge/powered%20by-tokio-blue?style=flat&logo=rust" alt="Powered by Tokio" />
  </a>
  <a href="https://rocket.rs">
    <img src="https://img.shields.io/badge/built%20with-rocket-red?style=flat&logo=rust" alt="Built with Rocket" />
  </a>
  <a href="https://ethereum.org">
    <img src="https://img.shields.io/badge/chain-ethereum-627EEA?style=flat&logo=ethereum" alt="Ethereum" />
  </a>
  <a href="https://base.org">
    <img src="https://img.shields.io/badge/chain-base-0052FF?style=flat" alt="Base" />
  </a>
  <a href="https://www.docker.com/">
    <img src="https://img.shields.io/badge/containerized-docker-2496ED?style=flat&logo=docker" alt="Docker" />
  </a>
  <br />
  <a href="https://github.com/rust-lang/rustfmt">
    <img src="https://img.shields.io/badge/code%20style-rustfmt-fc8d62?style=flat" alt="rustfmt" />
  </a>
  <a href="https://github.com/rust-lang/rust-clippy">
    <img src="https://img.shields.io/badge/linted%20with-clippy-ffc832?style=flat" alt="Clippy" />
  </a>
</p>

## What is Argus?

Argus is like a price comparison app for crypto traders. It watches the ETH/USDC price on two different exchanges:
- **Uniswap V4** on Ethereum (expensive but liquid)
- **Aerodrome** on Base (cheap but newer)

When prices differ, Argus calculates if you can make money by buying on the cheaper exchange and selling on the expensive one - even after paying transaction fees (gas).

**Example**: If ETH is $3,000 on Uniswap but $3,050 on Aerodrome, and gas costs total $30, you'd make $20 profit per ETH traded!

## Quick Start

### 1. Setup Environment

Create a `.env` file with these required settings:

```bash
# Required - Your RPC endpoints
ETHEREUM_RPC_URL=https://eth.llamarpc.com
BASE_RPC_URL=https://base.llamarpc.com

# Optional - Server configuration
SERVER_PORT=8080
CEX_PROVIDER=coinbase
```

### 2. Run Locally

```bash
# Install Rust if you haven't
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and run
git clone https://github.com/JumpiiX/argus
cd argus
cargo run
```

That's it! The service is now running at `http://localhost:8080`

## API Endpoint

### Check Arbitrage Opportunity

**GET** `/api/v1/arbitrage-opportunity?trade_size_eth=10`

**What you get back:**
```json
{
  "timestamp_utc": "2024-01-01T10:00:00Z",
  "trade_size_eth": 10.0,
  "reference_cex_price_usd": 3100.50,
  "uniswap_v4_details": {
    "effective_price_usd": 3098.25,
    "price_impact_percent": -0.072,
    "estimated_gas_cost_usd": 40.15
  },
  "aerodrome_details": {
    "effective_price_usd": 3105.75,
    "price_impact_percent": -0.150,
    "estimated_gas_cost_usd": 0.85
  },
  "arbitrage_summary": {
    "potential_profit_usd": 75.00,
    "total_gas_cost_usd": 41.00,
    "net_profit_usd": 34.00,
    "recommended_action": "ARBITRAGE_DETECTED"
  }
}
```

**What this means:**
- `effective_price_usd`: The actual price you'd get for your trade size
- `price_impact_percent`: How much your trade moves the market
- `estimated_gas_cost_usd`: Cost to execute the swap on that chain
- `net_profit_usd`: Your profit after all costs (if positive, there's an opportunity!)
- `recommended_action`: Either `ARBITRAGE_DETECTED` or `NO_ARBITRAGE`

### Health Check

**GET** `/health` - Returns `OK` if service is running

## How It Works (Behind the Scenes)

1. **Gets Reference Price**: Fetches ETH/USDC from Coinbase to know the "fair" market price
2. **Checks Both DEXs**: 
   - Asks Uniswap V4: "What's your ETH price?" 
   - Asks Aerodrome: "What's YOUR ETH price?"
3. **Calculates Real Costs**:
   - Ethereum gas: Like surge pricing during rush hour (can be $20-100 per transaction)
   - Base gas: Cheaper local fee + expensive Ethereum storage fee
4. **Does the Math**: 
   ```
   Profit = Price Difference × Amount
   Costs = Ethereum Gas + Base Gas
   Net = Profit - Costs
   
   If Net > 0: " ARBITRAGE OPPORTUNITY!"
   If Net ≤ 0: " Not profitable"
   ```

## Development Tools

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Run tests
cargo test

# Build optimized
cargo build --release
```

## Tech Stack

- **Runtime**: Tokio (async Rust) - Handles multiple operations at once
- **Web Framework**: Rocket - Simple, type-safe API endpoints
- **Blockchains**: Ethereum (expensive, established) + Base L2 (cheap, fast)
- **DEXs**: 
  - Uniswap V4: Advanced AMM with concentrated liquidity
  - Aerodrome: Simple constant-product AMM (x*y=k)
- **Price Feeds**: Coinbase, Kraken, Binance APIs for reference prices

## Project Structure

```
argus/
├── src/
│   ├── main.rs          # Entry point
│   ├── service.rs       # Core arbitrage logic
│   ├── api/             # REST endpoints
│   ├── rpc/             # Chain interactions
│   ├── dex/             # DEX integrations
│   ├── cex/             # CEX price feeds
│   └── analytics/       # Profit calculations
└── .env.example         # Config template
```

## License

MIT
