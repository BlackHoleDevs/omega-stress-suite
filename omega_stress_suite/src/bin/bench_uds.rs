use omega_bench_suite::benches::triple_uds::run_bench;
use omega_bench_suite::charts::generate_uds_chart;
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("⚔️  OMEGADRIVE 3.0 UDS RAW THROUGHPUT SHOWDOWN  ⚔️");
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

    let mut uds_results = Vec::new();

    // 1. Redis
    pb.set_message("⚡ Running Redis UDS Test...");
    let (rw, rr) = run_bench("Redis", "/tmp/redis_test.sock", num_clients, ops_per_client, batch_size, 64)
        .await
        .unwrap_or((0.0, 0.0));
    uds_results.push(("Redis", rw, rr));
    pb.inc(1);

    // 2. KeyDB
    pb.set_message("⚡ Running KeyDB UDS Test...");
    let (kw, kr) = run_bench("KeyDB", "/tmp/keydb.sock", num_clients, ops_per_client, batch_size, 64)
        .await
        .unwrap_or((0.0, 0.0));
    uds_results.push(("KeyDB", kw, kr));
    pb.inc(1);

    // 3. OmegaDrive
    pb.set_message("⚡ Running OmegaDrive UDS Test...");
    let (ow, or) = run_bench("OmegaDrive", "/tmp/airdb.sock", num_clients, ops_per_client, batch_size, 64)
        .await
        .unwrap_or((0.0, 0.0));
    uds_results.push(("OmegaDrive", ow, or));
    pb.inc(1);

    pb.finish_with_message("✅ UDS Benchmarks Completed Successfully!");

    println!("\n==================================================");
    println!("📈   PRINTING UDS SHOWDOWN COMPARISON METRICS   📈");
    println!("==================================================");
    println!("{:<15} | {:>15} | {:>15}", "Engine", "Write (ops/s)", "Read (ops/s)");
    println!("{:-<50}", "");
    for &(engine, w, r) in &uds_results {
        println!("{:<15} | {:>15.0} | {:>15.0}", engine, w, r);
    }
    println!("{:-<50}", "");

    println!("\n🎨 Generating stunning cyberpunk UDS performance chart...");
    match generate_uds_chart(&uds_results) {
        Ok(_) => println!("✅ Saved UDS showdown chart inside `plots/uds_performance.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE HAS DEFEATED THE COMPETITION! 🚀");
    println!("==================================================\n");
}
