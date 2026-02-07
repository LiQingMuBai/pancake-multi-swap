# PancakeSwap Multi-Account BNB → USDT Swapper (Rust Edition)

> **Automate secure, concurrent BNB-to-USDT swaps on BSC Mainnet across multiple wallets using Rust + Foundry.**

This tool leverages Rust’s memory safety and async runtime to execute token swaps via PancakeSwap V3's **Universal Router**. It supports multiple private keys, configurable slippage, and controlled concurrency — all while ensuring private keys never leak into logs or disk.

✅ **Memory-safe** – No risk of private key exposure  
✅ **Concurrent & efficient** – Tokio-based async execution  
✅ **Production-ready** – Structured logging, error isolation  
✅ **Single binary** – Easy deployment  


`
[package]
name = "uniswap-swap-rs"
version = "0.1.0"
edition = "2024"


[dependencies]
ethers = { version = "2.0", features = ["rustls"] }
tokio = { version = "1", features = ["full"] }
`


## 🔧 Requirements

- [Rust](https://www.rust-lang.org/) ≥ 1.91 (`cargo`, `rustc`)
- [Foundry](https://github.com/foundry-rs/foundry) (`forge` CLI installed)
- BSC-compatible RPC URL (e.g., `https://bsc-dataseed.binance.org`)
- Wallets funded with BNB (for gas + swap amount)

---

## 🚀 Quick Start

