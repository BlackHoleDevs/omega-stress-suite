use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::time::Instant;

pub async fn run_json_bench(
    name: &str,
    socket: &str,
    num_clients: usize,
    ops_per_client: usize,
    batch_size: usize,
    payload_size_kb: usize,
) -> Result<(f64, f64), String> {
    // Generate a structured JSON payload of the requested size
    let field_data = "x".repeat(payload_size_kb * 1024);
    let json_payload = serde_json::json!({
        "status": "success",
        "worker_pool": 16,
        "payload": field_data,
        "metadata": {
            "version": "1.0.0",
            "environment": "production"
        }
    }).to_string();

    let mut write_cmds = Vec::new();
    let mut read_cmds = Vec::new();
    
    for i in 0..batch_size {
        write_cmds.push(format!("*3\r\n$3\r\nSET\r\n$6\r\njson{:02}\r\n${}\r\n{}\r\n", i, json_payload.len(), json_payload));
        read_cmds.push(format!("*2\r\n$3\r\nGET\r\n$6\r\njson{:02}\r\n", i));
    }
    
    let write_cmd = write_cmds.join("");
    let read_cmd = read_cmds.join("");
    let total_ops = num_clients * ops_per_client;

    // WRITE BENCH
    let start_write = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..num_clients {
        let s_path = socket.to_string();
        let w_cmd = write_cmd.clone();
        let batches = ops_per_client / batch_size;
        workers.push(tokio::spawn(async move {
            let mut stream = loop {
                if let Ok(s) = UnixStream::connect(&s_path).await { break s; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            };
            let (rx, mut tx) = tokio::io::split(stream);
            let mut reader = BufReader::new(rx);
            for _ in 0..batches {
                tx.write_all(w_cmd.as_bytes()).await.unwrap();
                for _ in 0..batch_size {
                    let mut line = String::new();
                    reader.read_line(&mut line).await.unwrap();
                }
            }
        }));
    }
    for w in workers {
        w.await.map_err(|e| format!("Worker failed: {}", e))?;
    }
    let write_duration = start_write.elapsed().as_secs_f64();
    let write_ops = total_ops as f64 / write_duration;

    // READ BENCH
    let start_read = Instant::now();
    let mut workers = Vec::new();
    let json_len = json_payload.len();
    for _ in 0..num_clients {
        let s_path = socket.to_string();
        let r_cmd = read_cmd.clone();
        let batches = ops_per_client / batch_size;
        workers.push(tokio::spawn(async move {
            let mut stream = loop {
                if let Ok(s) = UnixStream::connect(&s_path).await { break s; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            };
            let mut body = vec![0u8; json_len + 2];
            let (rx, mut tx) = tokio::io::split(stream);
            let mut reader = BufReader::new(rx);
            for _ in 0..batches {
                tx.write_all(r_cmd.as_bytes()).await.unwrap();
                for _ in 0..batch_size {
                    let mut line = String::new();
                    reader.read_line(&mut line).await.unwrap(); // Read $len
                    reader.read_exact(&mut body).await.unwrap();
                }
            }
        }));
    }
    for w in workers {
        w.await.map_err(|e| format!("Worker failed: {}", e))?;
    }
    let read_duration = start_read.elapsed().as_secs_f64();
    let read_ops = total_ops as f64 / read_duration;

    Ok((write_ops, read_ops))
}
