use omega_bench_suite::benches::uds_batched::run_batched_bench;
use omega_bench_suite::charts::generate_uds_batched_chart;
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("⚔️  OMEGADRIVE 3.0 UDS BATCHED THROUGHPUT SHOWDOWN ⚔️");
    println!("==================================================\n");

    let num_clients = 64;
    let ops_per_client = 50000;
    let batch_size = 1000;

    let pb = ProgressBar::new(3);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut batched_results = Vec::new();

    // 1. Redis
    pb.set_message("⚡ Running Redis Batched Test...");
    let (rw, rr) = run_batched_bench("Redis", "/tmp/redis_test.sock", num_clients, ops_per_client, batch_size)
        .await
        .unwrap_or((0.0, 0.0));
    batched_results.push(("Redis", rw, rr));
    pb.inc(1);

    // 2. KeyDB
    pb.set_message("⚡ Running KeyDB Batched Test...");
    let (kw, kr) = run_batched_bench("KeyDB", "/tmp/keydb.sock", num_clients, ops_per_client, batch_size)
        .await
        .unwrap_or((0.0, 0.0));
    batched_results.push(("KeyDB", kw, kr));
    pb.inc(1);

    // 3. OmegaDrive
    pb.set_message("⚡ Running OmegaDrive Batched Test...");
    let (ow, or) = run_batched_bench("OmegaDrive", "/tmp/airdb.sock", num_clients, ops_per_client, batch_size)
        .await
        .unwrap_or((0.0, 0.0));
    batched_results.push(("OmegaDrive", ow, or));
    pb.inc(1);

    pb.finish_with_message("✅ UDS Batched Benchmarks Completed Successfully!");

    println!("\n==================================================");
    println!("📈   PRINTING BATCHED THROUGHPUT SHOWDOWN METRICS  📈");
    println!("==================================================");
    println!("{:<15} | {:>15} | {:>15}", "Engine", "MSET (ops/s)", "MGET (ops/s)");
    println!("{:-<50}", "");
    for &(engine, w, r) in &batched_results {
        println!("{:<15} | {:>15.0} | {:>15.0}", engine, w, r);
    }
    println!("{:-<50}", "");

    println!("\n🎨 Generating stunning cyberpunk batched performance chart...");
    match generate_uds_batched_chart(&batched_results) {
        Ok(_) => println!("✅ Saved batched UDS chart inside `plots/uds_batched_performance.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE HAS DEFEATED THE COMPETITION! 🚀");
    println!("==================================================\n");
}
