### Arrakis Finance - David Backend Engineer Coding Challenge

Below is an assessment of this repository against the provided coding challenge. Each category includes a rating and concise justification with references to relevant modules and implementation details.

## 1. Correctness & Functionality — Meets Expectations
- **What works**: 
  - REST API endpoint implemented with Rocket: `GET /api/v1/arbitrage-opportunity?trade_size_eth=...` in `src/api/mod.rs` returning a typed JSON payload (`ArbitrageOpportunity`).
  - Concurrent data fetching across three sources via `tokio::try_join!` in `src/service.rs`.
  - CEX price fetching implemented for Coinbase, Kraken, and Binance (`src/cex/*`), using public HTTP APIs.
  - DEX integrations provided for Uniswap V4 and Aerodrome with direct on-chain reads (`src/dex/uniswap_v4/pool.rs` and `src/dex/aerodrome/pool.rs`).
  - Gas cost estimation for Ethereum and Base, including Base L1 data fee via the OP L1 fee oracle (`src/service.rs` and `src/rpc/mod.rs`).
- **Gaps/risks**:
  - Uniswap V4 price-impact modeling is simplified and not tick/liquidity aware; effective price is derived by applying a fixed impact factor rather than simulating tick traversal (`src/dex/uniswap_v4/pool.rs`).
  - `price_impact_percent` units are inconsistent across DEXes: Aerodrome returns percent-like values (×100) from `utils::calculate_price_impact`, while Uniswap V4 uses a small decimal not scaled by 100.
  - Cross-chain transfer costs/latency beyond gas are not modeled.

Overall, the system runs and is directionally correct for Aerodrome and CEX. For Uniswap V4 the slippage modeling is approximate.

## 2. Code Quality & Idiomatic Rust — Meets Expectations
- **Strengths**:
  - Clear modular structure and layering: `api`, `service`, `rpc`, `dex`, `cex`, `analytics`, `utils`.
  - Errors modeled via `thiserror` and propagated with `Result` throughout (`src/models/mod.rs`).
  - Traits for abstraction (`DexClient`, `CexClient`) make the core service testable and swappable.
  - Structured logging via `tracing` and environment-driven filters (`src/main.rs`).
- **Concerns**:
  - A few `unwrap()`s remain (e.g., mutex lock, constant decimal parses) that could be handled explicitly.
  - Inconsistent percentage semantics between modules (see price impact notes).
  - Some dependencies (e.g., `config` crate, `prometheus`) are present but lightly used or not wired.
  - Using `ethers.rs` which is being deprecated in favor of `alloy.rs`

## 3. DeFi/EVM Domain Knowledge — Meets Expectations
- **Demonstrated**:
  - Understands V3/V4-style pricing via `sqrtPriceX96` and decodes spot price (`src/dex/uniswap_v4/pool.rs`; `src/utils/mod.rs`).
  - Implements reserves-based constant product math for Aerodrome and a realistic `getAmountOut`-style function (`src/dex/aerodrome/pool.rs`).
  - Models Base L2 fees including L1 data fee via the OP oracle (`src/rpc/mod.rs`).
- **Notably missing for “Exceeds”**:
  - No tick-based liquidity traversal or exact slippage modeling for Uniswap V4; price impact is a heuristic.
  - Fee parameters and percent scaling inconsistencies undermine accuracy.

## 4. System Architecture & Design — Meets Expectations
- **Positives**:
  - Clean separation of layers with dependency injection through traits; core logic focused in `src/service.rs`.
  - Concurrency with `tokio::try_join!` to minimize end-to-end latency; independent network calls executed in parallel.
  - Sensible error domains and typed models for responses.
  - K8s manifests and containerization support the service shape.
- **Areas to improve**:
  - No retries/backoff/circuit breaking around RPC/HTTP calls; a transient failure can fail the whole request.
  - No caching or rate limiting to protect upstreams or improve performance.
  - No configuration provider layering; environment-only with some unused config machinery.

## 5. Testing Strategy — Needs Improvement
- No unit or integration tests provided. Dev dependencies (`mockito`, `tokio-test`) exist but are unused.
- Critical functions like `sqrtPriceX96` conversion, `get_amount_out`, and gas/fee calculations lack tests, reducing confidence.

## 6. Documentation & Deployment — Meets Expectations
- **Documentation**: 
  - `README.md` is clear with setup, environment variables, and run instructions, including Docker usage and example responses.
  - `DESIGN_CHOICES.md` is thorough and articulates rationale, concurrency strategy, and production considerations.
- **Deployment**:
  - Dockerfile builds and runs, uses a non-root user and healthcheck. It is single-stage (larger image) rather than multi-stage.
  - `deployment.yaml` includes both Deployment and Service with basic probes and security context.
- **Minor gaps**:
  - Container image is not optimized (single-stage); exceeds expectations would require a lean multi-stage build and metrics endpoint wiring.

## Overall Assessment
- Solid submission that meets most functional and architectural expectations with good documentation. The primary gaps are rigorous slippage/price impact modeling for Uniswap V4, consistency of fee/percent semantics, lack of resiliency patterns, and absence of tests.

### Final Ratings
- Correctness & Functionality: Meets Expectations
- Code Quality & Idiomatic Rust: Meets Expectations
- DeFi/EVM Domain Knowledge: Meets Improvement
- System Architecture & Design: Meets Expectations
- Testing Strategy: Needs Improvement
- Documentation & Deployment: Meets Expectations
