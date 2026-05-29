# 🏎️ Omega Bench Suite — Next-Gen Database, Cache & Crypto Benchmarking Harness

[![Language: Rust](https://img.shields.io/badge/Language-Rust-orange.svg?logo=rust&style=flat-square)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![Framework: Tokio](https://img.shields.io/badge/Framework-Tokio-darkblue.svg?logo=tokio&style=flat-square)](https://tokio.rs)
[![Open Source](https://img.shields.io/badge/Open%20Source-%E2%9D%A4-red.svg?style=flat-square)](#)

**Omega Bench Suite** is an open-source, high-throughput asynchronous benchmarking harness engineered in Rust. Designed to stress-test next-generation in-memory databases, key-value stores, real-time WebSocket pub/sub brokers, and high-speed cryptographic ciphers under massive concurrent workloads.

This suite is the exact performance verification harness used to benchmark and validate **Omega Drive 3.0** against industry standards like Redis and KeyDB.

---

## 🔬 Benchmark Modules

The suite contains highly specialized asynchronous benchmark suites:

1. **`bench_tcp`**: Benchmarks raw multi-client TCP network round-trips (reads and writes) against target ports.
2. **`bench_uds` / `bench_uds_batched`**: Stress-tests zero-latency Unix Domain Sockets using both single-request and pipelined batching paradigms.
3. **`bench_ws`**: Spawns thousands of concurrent WebSocket subscribers and stress-tests pub/sub broadcasting throughput and latency distribution.
4. **`bench_crypto`**: Compares the dynamic performance of the **Dynamic Neural Cascade Cipher** (ChaCha20 + Neural XOR) against hardware-accelerated standard AES-256-GCM.
5. **`bench_json`**: Stress-tests database engines with complex structured JSON payloads (perfect for simulating e-commerce caching workloads).
6. **`bench_file`**: Benchmarks file-backed disk and page-cache persistent memory writing performance.

---

## ⚡ Key Architectural Advantages

* **Full Asynchronous Concurrency:** Built on **Tokio** to spawn thousands of lightweight virtual client tasks, simulating realistic high-concurrency production stress.
* **Auto-Generating Charts:** Integrates the **Plotters** visualization engine to parse raw throughput metrics and automatically output gorgeous, publication-ready PNG performance charts.
* **Zero-Overhead Statistics:** Utilizes high-precision monotonic clocks to capture sub-microsecond latencies without introducing profiling overhead.
* **Visual Progress:** Integrated with `indicatif` to render slick, real-time progress bars for each running benchmark.

---

## ⚙️ Quick Start

### 1. Prerequisites
Ensure you have the Rust toolchain installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Make sure you have your database engines running (e.g. `redis-server` on port `6379`, `keydb-server` on `6381`, and `omega` on `6380`) before executing network tests.

### 2. Running the Complete Suite & Chart Generation
To run all benchmarks sequentially and auto-generate the PNG charts in `/plots/`:
```bash
cargo run --release
```

This will run all tests and automatically produce beautiful throughput charts such as:
* `plots/tcp_performance.png`
* `plots/uds_performance.png`
* `plots/websocket_performance.png`
* `plots/crypto_performance.png`

### 3. Running Individual Benchmarks
You can execute specific benchmark binaries directly using Cargo's `--bin` flag:

```bash
# Stress-test TCP connections
cargo run --release --bin bench_tcp

# Stress-test UDS pipelining
cargo run --release --bin bench_uds_batched

# Benchmark WebSocket Pub/Sub
cargo run --release --bin bench_ws

# Compare Cryptographic Performance
cargo run --release --bin bench_crypto
```

---

## 📊 Sample Output & Visualization

When you execute the suite, it renders real-time performance progress and dumps structured statistics:

```text
[1/4] Running TCP Showdown...
  [========================================] 100k operations completed (5.9M ops/s)
[2/4] Running Cryptographic Showdown...
  [========================================] 1M encryptions completed (12.4M ops/s)

📊 Performance charts successfully saved to plots/ directory!
```

---

## 🛡️ License

This project is open-source and licensed under the **MIT License**. Feel free to use, modify, and distribute it in your own high-performance projects.

---

*Formulated with passion by the Omega Drive Creator.*
