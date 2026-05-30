use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::time::Instant;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;

pub async fn run_crypto_bench(
    socket: &str,
    num_clients: usize,
    ops_per_client: usize,
    batch_size: usize,
    use_aes: bool,
) -> Result<(f64, f64), String> {
    let raw_payload = "A".repeat(64); // 64-byte payload
    let total_ops = num_clients * ops_per_client;

    // Pre-initialize AES context if used
    let (aes_key, aes_nonce) = if use_aes {
        let key = Key::<Aes256Gcm>::from_slice(b"an-incredibly-secure-32-byte-key");
        let nonce = Nonce::from_slice(b"unique-nonce");
        (Some(key.clone()), Some(nonce.clone()))
    } else {
        (None, None)
    };

    // Encrypt the payload upfront for WRITE commands to simulate high-throughput client-side encryption
    let write_cmd = {
        let payload_to_send = if use_aes {
            let cipher = Aes256Gcm::new(aes_key.as_ref().unwrap());
            let ciphertext = cipher.encrypt(aes_nonce.as_ref().unwrap(), raw_payload.as_bytes())
                .map_err(|e| format!("AES Encryption failed: {:?}", e))?;
            hex::encode(ciphertext)
        } else {
            raw_payload.clone()
        };

        let mut write_cmds = Vec::new();
        for i in 0..batch_size {
            write_cmds.push(format!("*3\r\n$3\r\nSET\r\n$7\r\ncrypt{:02}\r\n${}\r\n{}\r\n", i, payload_to_send.len(), payload_to_send));
        }
        write_cmds.join("")
    };

    let read_cmd = {
        let mut read_cmds = Vec::new();
        for i in 0..batch_size {
            read_cmds.push(format!("*2\r\n$3\r\nGET\r\n$7\r\ncrypt{:02}\r\n", i));
        }
        read_cmds.join("")
    };

    // 1. WRITE BENCH
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
        w.await.map_err(|e| format!("Write worker failed: {}", e))?;
    }
    let write_duration = start_write.elapsed().as_secs_f64();
    let write_ops = total_ops as f64 / write_duration;

    // 2. READ BENCH (with client-side decryption if use_aes is true)
    let start_read = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..num_clients {
        let s_path = socket.to_string();
        let r_cmd = read_cmd.clone();
        let batches = ops_per_client / batch_size;
        
        let local_aes_key = aes_key.clone();
        let local_aes_nonce = aes_nonce.clone();

        workers.push(tokio::spawn(async move {
            let mut stream = loop {
                if let Ok(s) = UnixStream::connect(&s_path).await { break s; }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            };
            let (rx, mut tx) = tokio::io::split(stream);
            let mut reader = BufReader::new(rx);
            
            // Allocate decryption buffer if needed
            let cipher = local_aes_key.as_ref().map(|k| Aes256Gcm::new(k));
            
            for _ in 0..batches {
                tx.write_all(r_cmd.as_bytes()).await.unwrap();
                for _ in 0..batch_size {
                    let mut len_line = String::new();
                    reader.read_line(&mut len_line).await.unwrap(); // Read e.g. "$128\r\n"
                    
                    if len_line.starts_with('$') {
                        let len_str = len_line[1..].trim();
                        if let Ok(bytes_len) = len_str.parse::<usize>() {
                            let mut body = vec![0u8; bytes_len + 2]; // +2 for \r\n
                            reader.read_exact(&mut body).await.unwrap();
                            
                            if use_aes {
                                // Decrypt client-side
                                let hex_str = std::str::from_utf8(&body[..bytes_len]).unwrap();
                                let encrypted_bytes = hex::decode(hex_str).unwrap();
                                let decrypted = cipher.as_ref().unwrap()
                                    .decrypt(local_aes_nonce.as_ref().unwrap(), encrypted_bytes.as_slice())
                                    .unwrap();
                                assert_eq!(decrypted.len(), 64);
                            }
                        }
                    }
                }
            }
        }));
    }
    for w in workers {
        w.await.map_err(|e| format!("Read worker failed: {}", e))?;
    }
    let read_duration = start_read.elapsed().as_secs_f64();
    let read_ops = total_ops as f64 / read_duration;

    Ok((write_ops, read_ops))
}
