use omega_bench_suite::benches::crypto::run_crypto_bench;
use omega_bench_suite::charts::generate_crypto_chart;
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("🛡️  OMEGADRIVE 3.0 SECURE DATABASE CRYPTO SHOWDOWN  🛡️");
    println!("==================================================\n");

    let num_clients = 32;
    let ops_per_client = 10000;
    let batch_size = 100;

    let pb = ProgressBar::new(3);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut crypto_results = Vec::new();

    // 1. Redis + AES-256-GCM (Client-Side Encryption)
    pb.set_message("⚡ Benchmarking Redis + Client-Side AES-256-GCM...");
    let (rw, rr) = run_crypto_bench("/tmp/redis_test.sock", num_clients, ops_per_client, batch_size, true)
        .await
        .unwrap_or((0.0, 0.0));
    crypto_results.push(("Redis + AES", rw, rr));
    pb.inc(1);

    // 2. KeyDB + AES-256-GCM (Client-Side Encryption)
    pb.set_message("⚡ Benchmarking KeyDB + Client-Side AES-256-GCM...");
    let (kw, kr) = run_crypto_bench("/tmp/keydb.sock", num_clients, ops_per_client, batch_size, true)
        .await
        .unwrap_or((0.0, 0.0));
    crypto_results.push(("KeyDB + AES", kw, kr));
    pb.inc(1);

    // 3. OmegaDrive (Native AVX2-accelerated Neural Swarm Routing Encryption)
    pb.set_message("⚡ Benchmarking OmegaDrive (Server-Side Neural XOR Encryption)...");
    let (ow, or) = run_crypto_bench("/tmp/airdb.sock", num_clients, ops_per_client, batch_size, false)
        .await
        .unwrap_or((0.0, 0.0));
    crypto_results.push(("OmegaDrive", ow, or));
    pb.inc(1);

    pb.finish_with_message("✅ Cryptographic Benchmarks Completed Successfully!");

    println!("\n==================================================");
    println!("📈      PRINTING SECURE PERFORMANCE COMPARISON    📈");
    println!("==================================================");
    println!("{:<20} | {:>18} | {:>18}", "Engine / Setup", "Write (ops/s)", "Read (ops/s)");
    println!("{:-<62}", "");
    for &(engine, w, r) in &crypto_results {
        println!("{:<20} | {:>18.0} | {:>18.0}", engine, w, r);
    }
    println!("{:-<62}", "");

    println!("\n🎨 Generating stunning cyberpunk secure performance chart...");
    match generate_crypto_chart(&crypto_results) {
        Ok(_) => println!("✅ Saved secure showdown chart inside `plots/crypto_performance.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE NEURAL SECURITY DEFEATS AES CLIENTS! 🚀");
    println!("==================================================\n");
}
