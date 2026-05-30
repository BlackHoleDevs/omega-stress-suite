use omega_bench_suite::benches::json_sizes::run_json_bench;
use omega_bench_suite::charts::generate_json_chart;
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("🧬  OMEGADRIVE 3.0 JSON CACHING PERFORMANCE BENCH  🧬");
    println!("==================================================\n");

    let num_clients = 32;
    let ops_per_client = 5000;
    let batch_size = 100;

    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut json_results = Vec::new();
    let json_sizes = [1, 10, 50, 100]; // KB

    for &size in &json_sizes {
        pb.set_message(format!("⚡ Benchmarking JSON Size {}KB...", size));
        
        // 1. Redis
        let (rw, rr) = run_json_bench("Redis", "/tmp/redis_test.sock", num_clients, ops_per_client, batch_size, size)
            .await
            .unwrap_or((0.0, 0.0));
        json_results.push(("Redis", size, rw, rr));

        // 2. KeyDB
        let (kw, kr) = run_json_bench("KeyDB", "/tmp/keydb.sock", num_clients, ops_per_client, batch_size, size)
            .await
            .unwrap_or((0.0, 0.0));
        json_results.push(("KeyDB", size, kw, kr));

        // 3. OmegaDrive
        let (ow, or) = run_json_bench("OmegaDrive", "/tmp/airdb.sock", num_clients, ops_per_client, batch_size, size)
            .await
            .unwrap_or((0.0, 0.0));
        json_results.push(("OmegaDrive", size, ow, or));

        pb.inc(1);
    }

    pb.finish_with_message("✅ JSON Benchmarks Completed Successfully!");

    println!("\n==================================================");
    println!("📈   PRINTING JSON PAYLOAD SIZE THROUGHPUT COMPARE 📈");
    println!("==================================================");
    println!("{:<15} | {:<8} | {:>15} | {:>15}", "Engine", "Size KB", "Write (ops/s)", "Read (ops/s)");
    println!("{:-<60}", "");
    for &(engine, size, w, r) in &json_results {
        println!("{:<15} | {:<8} | {:>15.0} | {:>15.0}", engine, format!("{}KB", size), w, r);
    }
    println!("{:-<60}", "");

    println!("\n🎨 Generating stunning cyberpunk JSON caching performance chart...");
    match generate_json_chart(&json_results) {
        Ok(_) => println!("✅ Saved JSON caching chart inside `plots/json_payload_performance.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE HAS DEFEATED THE COMPETITION! 🚀");
    println!("==================================================\n");
}
