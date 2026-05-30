use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::time::Instant;

pub async fn run_file_bench(
    name: &str,
    socket: &str,
    num_clients: usize,
    ops_per_client: usize,
    batch_size: usize,
    chunk_size_mb: usize,
) -> Result<(f64, f64), String> {
    // Generate a raw binary vector payload
    let raw_payload = vec![120u8; chunk_size_mb * 1024 * 1024];
    
    // We cannot easily pass raw binary with standard formatting without escape or bulk formats,
    // so we format using standard RESP Bulk String format cleanly!
    let mut write_header = format!("*3\r\n$3\r\nSET\r\n$6\r\nfile00\r\n${}\r\n", raw_payload.len());
    let mut write_cmd = Vec::with_capacity(write_header.len() + raw_payload.len() + 2);
    write_cmd.extend_from_slice(write_header.as_bytes());
    write_cmd.extend_from_slice(&raw_payload);
    write_cmd.extend_from_slice(b"\r\n");

    let read_cmd = format!("*2\r\n$3\r\nGET\r\n$6\r\nfile00\r\n");
    let total_bytes_transferred = num_clients * ops_per_client * raw_payload.len();

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
                tx.write_all(&w_cmd).await.unwrap();
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
    // MB/s = bytes / 1024 / 1024 / seconds
    let write_mb_s = (total_bytes_transferred as f64) / (1024.0 * 1024.0) / write_duration;

    // READ BENCH
    let start_read = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..num_clients {
        let s_path = socket.to_string();
        let r_cmd = read_cmd.clone();
        let batches = ops_per_client / batch_size;
        let payload_len = raw_payload.len();
        workers.push(tokio::spawn(async move {
            let mut stream = loop {
                if let Ok(s) = UnixStream::connect(&s_path).await { break s; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            };
            let mut body = vec![0u8; payload_len + 2];
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
    let read_mb_s = (total_bytes_transferred as f64) / (1024.0 * 1024.0) / read_duration;

    Ok((write_mb_s, read_mb_s))
}
