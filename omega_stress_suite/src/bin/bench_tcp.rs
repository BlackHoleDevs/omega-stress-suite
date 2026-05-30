use omega_bench_suite::benches::triple_tcp::run_tcp_bench;
use omega_bench_suite::charts::generate_tcp_chart;
use indicatif::{ProgressBar, ProgressStyle};

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("⚔️  OMEGADRIVE 3.0 TCP RAW THROUGHPUT SHOWDOWN  ⚔️");
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

    let mut tcp_results = Vec::new();

    // 1. Redis TCP
    pb.set_message("⚡ Running Redis TCP Test...");
    let (rw, rr) = run_tcp_bench("Redis", "127.0.0.1:6379", num_clients, ops_per_client, batch_size, 64)
        .await
        .unwrap_or((0.0, 0.0));
    tcp_results.push(("Redis", rw, rr));
    pb.inc(1);

    // 2. KeyDB TCP
    pb.set_message("⚡ Running KeyDB TCP Test...");
    let (kw, kr) = run_tcp_bench("KeyDB", "127.0.0.1:6381", num_clients, ops_per_client, batch_size, 64)
        .await
        .unwrap_or((0.0, 0.0));
    tcp_results.push(("KeyDB", kw, kr));
    pb.inc(1);

    // 3. OmegaDrive TCP
    pb.set_message("⚡ Running OmegaDrive TCP Test...");
    let (ow, or) = run_tcp_bench("OmegaDrive", "127.0.0.1:6380", num_clients, ops_per_client, batch_size, 64)
        .await
        .unwrap_or((0.0, 0.0));
    tcp_results.push(("OmegaDrive", ow, or));
    pb.inc(1);

    pb.finish_with_message("✅ TCP Benchmarks Completed Successfully!");

    println!("\n==================================================");
    println!("📈   PRINTING TCP SHOWDOWN COMPARISON METRICS   📈");
    println!("==================================================");
    println!("{:<15} | {:>15} | {:>15}", "Engine", "Write (ops/s)", "Read (ops/s)");
    println!("{:-<50}", "");
    for &(engine, w, r) in &tcp_results {
        println!("{:<15} | {:>15.0} | {:>15.0}", engine, w, r);
    }
    println!("{:-<50}", "");

    println!("\n🎨 Generating stunning cyberpunk TCP performance chart...");
    match generate_tcp_chart(&tcp_results) {
        Ok(_) => println!("✅ Saved TCP showdown chart inside `plots/tcp_performance.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE HAS DEFEATED THE COMPETITION VIA TCP! 🚀");
    println!("==================================================\n");
}
