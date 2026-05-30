use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt};
use std::time::Instant;

pub async fn run_batched_bench(
    name: &str,
    socket: &str,
    num_clients: usize,
    ops_per_client: usize,
    batch_size: usize,
) -> Result<(f64, f64), String> {
    let mut workers = Vec::with_capacity(num_clients);

    for id in 0..num_clients {
        let s_path = socket.to_string();
        workers.push(tokio::spawn(async move {
            let mut stream = loop {
                if let Ok(s) = UnixStream::connect(&s_path).await { break s; }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            };

            let mut reader = BufReader::new(stream);
            let mut mset_cmds = Vec::with_capacity(ops_per_client / batch_size);
            let mut mget_cmds = Vec::with_capacity(ops_per_client / batch_size);

            for i in (0..ops_per_client).step_by(batch_size) {
                let mut mset = format!("*{}\r\n$4\r\nMSET\r\n", batch_size * 2 + 1);
                let mut mget = format!("*{}\r\n$4\r\nMGET\r\n", batch_size + 1);
                for j in 0..batch_size {
                    let key = format!("c{}_k{}", id, i + j);
                    let val = "val64_0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
                    mset.push_str(&format!("${}\r\n{}\r\n${}\r\n{}\r\n", key.len(), key, val.len(), val));
                    mget.push_str(&format!("${}\r\n{}\r\n", key.len(), key));
                }
                mset_cmds.push(mset);
                mget_cmds.push(mget);
            }

            // 1. MSET
            let start_write = Instant::now();
            for cmd in &mset_cmds {
                let stream_ref = reader.get_mut();
                stream_ref.write_all(cmd.as_bytes()).await.unwrap();
                stream_ref.flush().await.unwrap();
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
            }
            let write_dur = start_write.elapsed().as_secs_f64();

            // 2. MGET
            let start_read = Instant::now();
            for cmd in &mget_cmds {
                let stream_ref = reader.get_mut();
                stream_ref.write_all(cmd.as_bytes()).await.unwrap();
                stream_ref.flush().await.unwrap();
                read_resp_array(&mut reader).await;
            }
            let read_dur = start_read.elapsed().as_secs_f64();

            (write_dur, read_dur)
        }));
    }

    let mut total_write_time = 0.0;
    let mut total_read_time = 0.0;

    for w in workers {
        let (write_dur, read_dur) = w.await.map_err(|e| format!("Worker failed: {}", e))?;
        total_write_time += write_dur;
        total_read_time += read_dur;
    }

    // Average client throughput
    let avg_write_duration = total_write_time / num_clients as f64;
    let avg_read_duration = total_read_time / num_clients as f64;

    let total_ops = num_clients * ops_per_client;
    let write_tps = total_ops as f64 / avg_write_duration;
    let read_tps = total_ops as f64 / avg_read_duration;

    Ok((write_tps, read_tps))
}

async fn read_resp_array<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) {
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    if !line.starts_with('*') { return; }
    let count: usize = line[1..].trim().parse().unwrap();
    
    for _ in 0..count {
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        if line.starts_with("$-1") { continue; }
        let bulk_len: usize = line[1..].trim().parse().unwrap();
        let mut data = vec![0u8; bulk_len + 2];
        reader.read_exact(&mut data).await.unwrap();
    }
}
