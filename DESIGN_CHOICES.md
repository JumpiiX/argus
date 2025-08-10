# Design Choices and Implementation Decisions

## Understanding the Task

### The Challenge in Simple Terms

Imagine you're at two different markets (Uniswap V4 on Ethereum and Aerodrome on Base) where people trade digital assets, specifically WETH (Wrapped Ethereum) for USDC (US Dollar Coin). Sometimes, due to different supply and demand at each market, the same asset might have different prices. This creates an opportunity: you could theoretically buy cheap at one market and sell high at another, making a profit from the price difference.

However, there's a catch - moving between these markets costs money (gas fees for blockchain transactions). The challenge is to build a service that:
1. Monitors prices at both markets continuously
2. Calculates if the price difference is large enough to cover transaction costs
3. Alerts when a profitable opportunity exists

### My Understanding of the Requirements

The task requires building a production-ready Rust service that monitors WETH/USDC price discrepancies across two different blockchain networks to identify arbitrage opportunities. The service must:

- Connect to Uniswap V4 on Ethereum Mainnet
- Connect to Aerodrome Finance on Base Mainnet  
- Fetch reference prices from a centralized exchange (CEX)
- Calculate effective execution prices considering slippage
- Factor in gas costs for both chains
- Expose findings via a REST API

## Initial Analysis and First Steps

### Protocol Version Investigation

My first critical step was to understand the fundamental differences between the two DEXs:

**Uniswap V4 Discovery:**
- Uniswap V4 uses a revolutionary **singleton pattern** with a single PoolManager contract (`0x000000000004444c5dc75cb358380d2e3de08a90`)
- Unlike V3 where each pool is a separate contract, V4 manages all pools through one contract
- Pools are identified using a PoolKey structure containing: currency0, currency1, fee, tickSpacing, and hooks
- This is a concentrated liquidity AMM using tick-based pricing

**Aerodrome Finance Discovery:**
- Aerodrome is a **Solidly/Velodrome fork**, fundamentally a V2-style AMM
- Uses the traditional constant product formula (x * y = k) for volatile pools
- Uses Curve's stableswap invariant for stable pools
- Has individual pool contracts rather than a singleton
- The WETH/USDC pool address: `0xcDAC0d6c6C59727a65F871236188350531885C43`

### Key Architectural Differences Identified

The fundamental difference between these protocols shaped my entire approach:

| Aspect | Uniswap V4 | Aerodrome |
|--------|------------|-----------|
| Architecture | Singleton PoolManager | Individual pool contracts |
| Liquidity Model | Concentrated (tick-based) | Full-range (reserves-based) |
| Price Storage | sqrtPriceX96 format | Direct reserve ratios |
| Pool Identification | PoolKey → Pool ID hash | Direct contract address |
| State Reading | getSlot0() returns price, tick, fees | getReserves() returns token amounts |

### Async-First Design

Since the service needs to fetch data from three independent sources (Ethereum RPC, Base RPC, and CEX API), I immediately recognized this as a perfect use case for concurrent operations. The decision to use Tokio and async/await throughout was driven by:

1. **Network I/O Bound**: Most operations wait for network responses
2. **Independent Data Sources**: No dependency between the three data fetches
3. **Real-time Requirements**: Need to minimize latency for opportunity detection

---

*Note: The following sections will detail the specific technology choices, implementation details, and production considerations...*