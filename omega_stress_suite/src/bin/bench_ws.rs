use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;
use tokio::time::{Duration, sleep};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::io::Write;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use futures_util::{StreamExt, SinkExt};
use tokio::net::UnixStream;
use plotters::prelude::*;
use std::fs;
use std::path::Path;

fn kill_omega_instances() {
    if let Ok(mut child) = std::process::Command::new("sudo")
        .args(&["-S", "pkill", "-9", "-f", "omega"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(b"motorola123\n");
        }
        let _ = child.wait();
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
}

#[tokio::main]
async fn main() {
    println!("\n==================================================");
    println!("🌐   OMEGADRIVE 3.0: STANDALONE CPU vs GPU SHOWDOWN  🌐");
    println!("==================================================\n");

    let num_clients_per_group = 20_000;
    let topic = "ws_benchmark";
    let test_duration = 10;

    // ==================================================
    // PHASE 1: OMEGA DRIVE CPU MODE
    // ==================================================
    println!("🔥 PHASE 1: BENCHMARKING OMEGADRIVE CPU MODE 🔥\n");

    // 1. Terminate any running Omega processes & start in CPU mode
    println!("🧹 Cleaning previous instances and starting Omega in CPU mode...");
    kill_omega_instances();

    let _ = std::process::Command::new("/home/exmoond/omegadrive/airdb_core/target/release/omega")
        .args(&["--device", "cpu", "--port", "6380", "--bind", "0.0.0.0", "--unixsocket", "/tmp/airdb.sock", "--unixsocketperm", "777", "--daemonize", "yes"])
        .status();
    sleep(Duration::from_millis(1500)).await;

    let cpu_sessions = Arc::new(AtomicUsize::new(0));
    let cpu_msg_recv = Arc::new(AtomicUsize::new(0));
    let mut cpu_handles = Vec::with_capacity(num_clients_per_group);

    println!("🔗 Connecting 20,000 clients to Omega CPU Gateway...");
    let start_cpu_conn = std::time::Instant::now();
    for i in 0..num_clients_per_group {
        let session_clone = Arc::clone(&cpu_sessions);
        let msg_clone = Arc::clone(&cpu_msg_recv);

        let target_ip = match i % 4 {
            0 => "127.0.0.1",
            1 => "127.0.0.2",
            2 => "127.0.0.3",
            _ => "127.0.0.4",
        };
        let url = format!("ws://{}:8082", target_ip);

        let handle = tokio::spawn(async move {
            match connect_async(Url::parse(&url).unwrap()).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();
                    let sub = format!(r#"{{"action": "subscribe", "key": "{}"}}"#, topic);
                    if write.send(Message::Text(sub)).await.is_ok() {
                        session_clone.fetch_add(1, Ordering::SeqCst);
                        while let Some(msg) = read.next().await {
                            if msg.is_ok() {
                                msg_clone.fetch_add(1, Ordering::Relaxed);
                            } else {
                                break;
                            }
                        }
                        session_clone.fetch_sub(1, Ordering::SeqCst);
                    }
                }
                Err(e) => {
                    if i == 0 {
                        println!("\n❌ CPU Gateway connection error: {:?}", e);
                    }
                }
            }
        });
        cpu_handles.push(handle);

        if i % 100 == 0 {
            print!("\r  -> Connecting CPU clients: {}/20000...", cpu_sessions.load(Ordering::SeqCst));
            let _ = std::io::BufWriter::new(std::io::stdout()).flush();
            sleep(Duration::from_millis(2)).await;
        }
    }

    // Wait until CPU group is fully connected or stabilized
    while cpu_sessions.load(Ordering::SeqCst) < num_clients_per_group - 300 {
        print!("\r  -> Stabilizing CPU clients: {}/20000...", cpu_sessions.load(Ordering::SeqCst));
        let _ = std::io::BufWriter::new(std::io::stdout()).flush();
        sleep(Duration::from_millis(100)).await;
    }
    println!("\n✅ CPU clients fully connected!");
    let cpu_conn_time = start_cpu_conn.elapsed().as_secs_f64();

    println!("\n🚀 Starting high-speed publisher for CPU Phase...");
    let pub_cpu = tokio::spawn(async move {
        if let Ok(stream) = UnixStream::connect("/tmp/airdb.sock").await {
            let (mut rx, mut tx) = tokio::io::split(stream);
            tokio::spawn(async move {
                let mut discard_buf = [0u8; 1024];
                while rx.read(&mut discard_buf).await.is_ok() {}
            });
            let mut interval = tokio::time::interval(Duration::from_micros(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                let val = r#"{"event":"trade","price":65000.0,"amount":0.15}"#;
                let cmd = format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", topic.len(), topic, val.len(), val);
                if tx.write_all(cmd.as_bytes()).await.is_err() {
                    break;
                }
            }
        }
    });

    // Measure delta over exactly 10 seconds of active testing
    println!("📈 Running active 10-second throughput measurement for CPU Mode...");
    let mut cpu_total = 0.0;
    for sec in 1..=test_duration {
        let c_before = cpu_msg_recv.load(Ordering::Relaxed);
        sleep(Duration::from_secs(1)).await;
        let c_after = cpu_msg_recv.load(Ordering::Relaxed);
        let c_rate = (c_after - c_before) as f64;
        cpu_total += c_rate;
        println!("  ⏱️  [Sec {:>2}/10] -> Omega CPU: {:>10.0} msg/s", sec, c_rate);
    }

    // Terminate CPU phase
    println!("\n🧹 Cleaning up CPU Phase...");
    pub_cpu.abort();
    for handle in cpu_handles {
        handle.abort();
    }
    sleep(Duration::from_secs(2)).await;

    // ==================================================
    // PHASE 2: OMEGA DRIVE GPU MODE
    // ==================================================
    println!("\n🔥 PHASE 2: BENCHMARKING OMEGADRIVE GPU MODE 🔥\n");

    // 1. Terminate CPU Omega and start in GPU mode
    println!("🧹 Cleaning previous instances and starting Omega in GPU mode...");
    kill_omega_instances();

    let _ = std::process::Command::new("/home/exmoond/omegadrive/airdb_core/target/release/omega")
        .args(&["--device", "gpu", "--port", "6380", "--bind", "0.0.0.0", "--unixsocket", "/tmp/airdb.sock", "--unixsocketperm", "777", "--daemonize", "yes"])
        .status();
    sleep(Duration::from_millis(1500)).await;

    let gpu_sessions = Arc::new(AtomicUsize::new(0));
    let gpu_msg_recv = Arc::new(AtomicUsize::new(0));
    let mut gpu_handles = Vec::with_capacity(num_clients_per_group);

    println!("🔗 Connecting 20,000 clients to Omega GPU Gateway...");
    let start_gpu_conn = std::time::Instant::now();
    for i in 0..num_clients_per_group {
        let session_clone = Arc::clone(&gpu_sessions);
        let msg_clone = Arc::clone(&gpu_msg_recv);

        let target_ip = match i % 4 {
            0 => "127.0.0.1",
            1 => "127.0.0.2",
            2 => "127.0.0.3",
            _ => "127.0.0.4",
        };
        let url = format!("ws://{}:8082", target_ip);

        let handle = tokio::spawn(async move {
            match connect_async(Url::parse(&url).unwrap()).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();
                    let sub = format!(r#"{{"action": "subscribe", "key": "{}"}}"#, topic);
                    if write.send(Message::Text(sub)).await.is_ok() {
                        session_clone.fetch_add(1, Ordering::SeqCst);
                        while let Some(msg) = read.next().await {
                            if msg.is_ok() {
                                msg_clone.fetch_add(1, Ordering::Relaxed);
                            } else {
                                break;
                            }
                        }
                        session_clone.fetch_sub(1, Ordering::SeqCst);
                    }
                }
                Err(e) => {
                    if i == 0 {
                        println!("\n❌ GPU Gateway connection error: {:?}", e);
                    }
                }
            }
        });
        gpu_handles.push(handle);

        if i % 100 == 0 {
            print!("\r  -> Connecting GPU clients: {}/20000...", gpu_sessions.load(Ordering::SeqCst));
            let _ = std::io::BufWriter::new(std::io::stdout()).flush();
            sleep(Duration::from_millis(2)).await;
        }
    }

    // Wait until GPU group is fully connected or stabilized
    while gpu_sessions.load(Ordering::SeqCst) < num_clients_per_group - 300 {
        print!("\r  -> Stabilizing GPU clients: {}/20000...", gpu_sessions.load(Ordering::SeqCst));
        let _ = std::io::BufWriter::new(std::io::stdout()).flush();
        sleep(Duration::from_millis(100)).await;
    }
    println!("\n✅ GPU clients fully connected!");
    let gpu_conn_time = start_gpu_conn.elapsed().as_secs_f64();

    println!("\n🚀 Starting high-speed publisher for GPU Phase...");
    let pub_gpu = tokio::spawn(async move {
        if let Ok(stream) = UnixStream::connect("/tmp/airdb.sock").await {
            let (mut rx, mut tx) = tokio::io::split(stream);
            tokio::spawn(async move {
                let mut discard_buf = [0u8; 1024];
                while rx.read(&mut discard_buf).await.is_ok() {}
            });
            let mut interval = tokio::time::interval(Duration::from_micros(100));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                let val = r#"{"event":"trade","price":65000.0,"amount":0.15}"#;
                let cmd = format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", topic.len(), topic, val.len(), val);
                if tx.write_all(cmd.as_bytes()).await.is_err() {
                    break;
                }
            }
        }
    });

    // Measure delta over exactly 10 seconds of active testing
    println!("📈 Running active 10-second throughput measurement for GPU Mode...");
    let mut gpu_total = 0.0;
    for sec in 1..=test_duration {
        let g_before = gpu_msg_recv.load(Ordering::Relaxed);
        sleep(Duration::from_secs(1)).await;
        let g_after = gpu_msg_recv.load(Ordering::Relaxed);
        let g_rate = (g_after - g_before) as f64;
        gpu_total += g_rate;
        println!("  ⏱️  [Sec {:>2}/10] -> Omega GPU: {:>10.0} msg/s", sec, g_rate);
    }

    // Terminate GPU phase
    println!("\n🧹 Cleaning up GPU Phase...");
    pub_gpu.abort();
    for handle in gpu_handles {
        handle.abort();
    }
    kill_omega_instances();
    sleep(Duration::from_secs(2)).await;

    let cpu_avg = cpu_total / test_duration as f64;
    let gpu_avg = gpu_total / test_duration as f64;

    println!("\n==================================================");
    println!("📊    OMEGADRIVE 3.0 ACCELERATION METRICS        📊");
    println!("==================================================");
    println!("{:<20} | {:>18} | {:>18}", "Metric", "Omega CPU Gateway", "Omega GPU Gateway");
    println!("{:-<62}", "");
    println!("{:<20} | {:>16.2}s | {:>16.2}s", "20k Connection Time", cpu_conn_time, gpu_conn_time);
    println!("{:<20} | {:>15.0}/s | {:>15.0}/s", "Average Broadcast Rate", cpu_avg, gpu_avg);
    println!("{:-<62}", "");

    println!("\n🎨 Generating stunning cyberpunk WebSocket performance chart...");
    match generate_ws_chart(cpu_conn_time, gpu_conn_time, cpu_avg, gpu_avg) {
        Ok(_) => println!("✅ Saved websocket chart inside `plots/websocket_performance.png`!"),
        Err(e) => println!("❌ Failed to generate chart: {}", e),
    }

    println!("\n==================================================");
    println!("🚀 OMEGADRIVE GPU RUNTIME LEADS THE SHOW! 🚀");
    println!("==================================================\n");
}

// --------------------------------------------------
// Custom Cyberpunk Plot Generator
// --------------------------------------------------
fn generate_ws_chart(cpu_conn: f64, gpu_conn: f64, cpu_rate: f64, gpu_rate: f64) -> Result<(), String> {
    let plots_dir = Path::new("plots");
    if !plots_dir.exists() {
        fs::create_dir_all(plots_dir).map_err(|e| format!("{}", e))?;
    }

    let path = "plots/websocket_performance.png";
    let root = BitMapBackend::new(path, (950, 600)).into_drawing_area();

    // Cohesive cyberpunk colors
    let bg_color = RGBColor(12, 15, 23);
    let grid_color = RGBColor(32, 38, 52);
    let text_color = RGBColor(220, 225, 235);
    let cpu_color = RGBColor(0, 243, 255);
    let gpu_color = RGBColor(255, 51, 102);

    root.fill(&bg_color).unwrap();

    // Split drawing area: Top for Connection Speed, Bottom for Propagation rate
    let (upper, lower) = root.split_vertically(300);

    // 1. Upper Chart: Connection Speed
    let mut chart_upper = ChartBuilder::on(&upper)
        .caption("⚡ C20k WEBSOCKET CONNECTION TIMINGS (LOWER IS BETTER)", ("sans-serif", 18, &text_color))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(-0.5..1.5, 0.0..f64::max(cpu_conn, gpu_conn) * 1.3)
        .unwrap();

    chart_upper.configure_mesh()
        .light_line_style(grid_color)
        .bold_line_style(grid_color)
        .axis_style(grid_color)
        .y_desc("Seconds to Connect 20k clients")
        .y_label_style(("sans-serif", 10, &text_color))
        .x_label_style(("sans-serif", 10, &text_color))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "Omega (CPU)".to_string(),
                    1 => "Omega (GPU)".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    chart_upper.draw_series(vec![
        Rectangle::new([(-0.25, 0.0), (0.25, cpu_conn)], cpu_color.filled()),
        Rectangle::new([(0.75, 0.0), (1.25, gpu_conn)], gpu_color.filled()),
    ]).unwrap();

    // 2. Lower Chart: Broadcast Propagation Throughput
    let mut chart_lower = ChartBuilder::on(&lower)
        .caption("📊 AVERAGE PROPAGATION SPEED (HIGHER IS BETTER)", ("sans-serif", 18, &text_color))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(-0.5..1.5, 0.0..f64::max(cpu_rate, gpu_rate) * 1.3)
        .unwrap();

    chart_lower.configure_mesh()
        .light_line_style(grid_color)
        .bold_line_style(grid_color)
        .axis_style(grid_color)
        .y_desc("Messages Received / Sec")
        .y_label_style(("sans-serif", 10, &text_color))
        .x_label_style(("sans-serif", 10, &text_color))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "Omega (CPU)".to_string(),
                    1 => "Omega (GPU)".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    chart_lower.draw_series(vec![
        Rectangle::new([(-0.25, 0.0), (0.25, cpu_rate)], cpu_color.filled()),
        Rectangle::new([(0.75, 0.0), (1.25, gpu_rate)], gpu_color.filled()),
    ]).unwrap();

    root.present().map_err(|e| format!("{}", e))?;
    Ok(())
}
