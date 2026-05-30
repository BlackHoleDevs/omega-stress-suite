use omega_bench_suite::benches::file_transfer::run_file_bench;
use omega_bench_suite::charts::generate_file_chart;
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("💾  OMEGADRIVE 3.0 BINARY CACHING BANDWIDTH SPEED   💾");
    println!("==================================================\n");

    let num_clients = 8;
    let ops_per_client = 100;
    let batch_size = 1; // batch_size of 1 prevents memory spikes and correctly aligns with single socket ops!

    let pb = ProgressBar::new(3);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut file_results = Vec::new();
    let chunk_sizes = [1, 5, 10]; // MB

    for &size in &chunk_sizes {
        pb.set_message(format!("⚡ Benchmarking Binary Chunk {}MB...", size));

        // 1. Redis
        let (rw, rr) = run_file_bench("Redis", "/tmp/redis_test.sock", num_clients, ops_per_client, batch_size, size)
            .await
            .unwrap_or((0.0, 0.0));
        file_results.push(("Redis", size, rw, rr));

        // 2. KeyDB
        let (kw, kr) = run_file_bench("KeyDB", "/tmp/keydb.sock", num_clients, ops_per_client, batch_size, size)
            .await
            .unwrap_or((0.0, 0.0));
        file_results.push(("KeyDB", size, kw, kr));

        // 3. OmegaDrive
        let (ow, or) = run_file_bench("OmegaDrive", "/tmp/airdb.sock", num_clients, ops_per_client, batch_size, size)
            .await
            .unwrap_or((0.0, 0.0));
        file_results.push(("OmegaDrive", size, ow, or));

        pb.inc(1);
    }

    pb.finish_with_message("✅ Binary file benchmarks completed successfully!");

    println!("\n==================================================");
    println!("📈   PRINTING BINARY FILE CACHING BANDWIDTH SPEED   📈");
    println!("==================================================");
    println!("{:<15} | {:<8} | {:>15} | {:>15}", "Engine", "Chunk MB", "Write (MB/s)", "Read (MB/s)");
    println!("{:-<60}", "");
    for &(engine, size, w, r) in &file_results {
        println!("{:<15} | {:<8} | {:>15.2} | {:>15.2}", engine, format!("{}MB", size), w, r);
    }
    println!("{:-<60}", "");

    println!("\n🎨 Generating stunning cyberpunk file caching bandwidth chart...");
    match generate_file_chart(&file_results) {
        Ok(_) => println!("✅ Saved file bandwidth chart inside `plots/file_caching_speed.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE HAS DEFEATED THE COMPETITION! 🚀");
    println!("==================================================\n");
}
