use plotters::prelude::*;
use std::fs;
use std::path::Path;

fn ensure_plots_dir() -> Result<(), String> {
    let plots_dir = Path::new("plots");
    if !plots_dir.exists() {
        fs::create_dir_all(plots_dir).map_err(|e| format!("Failed to create plots dir: {}", e))?;
    }
    Ok(())
}

// Global cohesive cyberpunk design tokens
const BG_COLOR: RGBColor = RGBColor(12, 15, 23);       // Dark blue-black #0c0f17
const GRID_COLOR: RGBColor = RGBColor(32, 38, 52);     // Muted gray #202634
const TEXT_COLOR: RGBColor = RGBColor(220, 225, 235);  // off-white
const OMEGA_COLOR: RGBColor = RGBColor(0, 243, 255);   // Neon Cyan
const REDIS_COLOR: RGBColor = RGBColor(255, 51, 102);  // Neon Crimson
const KEYDB_COLOR: RGBColor = RGBColor(161, 85, 232);  // Muted Purple

// ==================================================
// 1. UDS PERFORMANCE CHART (plots/uds_performance.png)
// ==================================================
pub fn generate_uds_chart(uds_results: &[(&str, f64, f64)]) -> Result<(), String> {
    ensure_plots_dir()?;
    let path = "plots/uds_performance.png";
    let root = BitMapBackend::new(path, (900, 600)).into_drawing_area();
    root.fill(&BG_COLOR).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("⚔️ UDS SHOWDOWN: RAW THROUGHPUT (32 CLIENTS)", ("sans-serif", 24, &TEXT_COLOR))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            -0.5..5.5,
            0.0..10_000_000.0,
        )
        .unwrap();

    chart.configure_mesh()
        .light_line_style(GRID_COLOR)
        .bold_line_style(GRID_COLOR)
        .axis_style(GRID_COLOR)
        .y_desc("Operations per Second")
        .y_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_style(("sans-serif", 10, &TEXT_COLOR))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "Redis-W".to_string(),
                    1 => "Redis-R".to_string(),
                    2 => "KeyDB-W".to_string(),
                    3 => "KeyDB-R".to_string(),
                    4 => "OmegaDrive-W".to_string(),
                    5 => "OmegaDrive-R".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    // Map engines to scores
    let mut scores = vec![0.0; 6];
    for &(engine, write, read) in uds_results {
        match engine {
            "Redis" => {
                scores[0] = write;
                scores[1] = read;
            }
            "KeyDB" => {
                scores[2] = write;
                scores[3] = read;
            }
            "OmegaDrive" => {
                scores[4] = write;
                scores[5] = read;
            }
            _ => {}
        }
    }

    // Draw bars centered at each index with a width of 0.7
    chart.draw_series(
        scores.iter().enumerate().map(|(idx, &val)| {
            let color = match idx {
                0 | 1 => REDIS_COLOR,
                2 | 3 => KEYDB_COLOR,
                _ => OMEGA_COLOR,
            };
            Rectangle::new(
                [
                    (idx as f64 - 0.35, 0.0),
                    (idx as f64 + 0.35, val),
                ],
                color.filled(),
            )
        })
    ).unwrap();

    root.present().map_err(|e| format!("Failed to save chart: {}", e))?;
    Ok(())
}

// ==================================================
// 2. UDS BATCHED MSET/MGET SHOWDOWN (plots/uds_batched_performance.png)
// ==================================================
pub fn generate_uds_batched_chart(uds_results: &[(&str, f64, f64)]) -> Result<(), String> {
    ensure_plots_dir()?;
    let path = "plots/uds_batched_performance.png";
    let root = BitMapBackend::new(path, (900, 600)).into_drawing_area();
    root.fill(&BG_COLOR).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("⚔️ BATCHED UDS SHOWDOWN: MSET/MGET THROUGHPUT (64 CLIENTS)", ("sans-serif", 24, &TEXT_COLOR))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            -0.5..5.5,
            0.0..30_000_000.0,
        )
        .unwrap();

    chart.configure_mesh()
        .light_line_style(GRID_COLOR)
        .bold_line_style(GRID_COLOR)
        .axis_style(GRID_COLOR)
        .y_desc("Operations per Second")
        .y_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_style(("sans-serif", 10, &TEXT_COLOR))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "Redis-MSET".to_string(),
                    1 => "Redis-MGET".to_string(),
                    2 => "KeyDB-MSET".to_string(),
                    3 => "KeyDB-MGET".to_string(),
                    4 => "Omega-MSET".to_string(),
                    5 => "Omega-MGET".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    // Map engines to scores
    let mut scores = vec![0.0; 6];
    for &(engine, write, read) in uds_results {
        match engine {
            "Redis" => {
                scores[0] = write;
                scores[1] = read;
            }
            "KeyDB" => {
                scores[2] = write;
                scores[3] = read;
            }
            "OmegaDrive" => {
                scores[4] = write;
                scores[5] = read;
            }
            _ => {}
        }
    }

    chart.draw_series(
        scores.iter().enumerate().map(|(idx, &val)| {
            let color = match idx {
                0 | 1 => REDIS_COLOR,
                2 | 3 => KEYDB_COLOR,
                _ => OMEGA_COLOR,
            };
            Rectangle::new(
                [
                    (idx as f64 - 0.35, 0.0),
                    (idx as f64 + 0.35, val),
                ],
                color.filled(),
            )
        })
    ).unwrap();

    root.present().map_err(|e| format!("Failed to save chart: {}", e))?;
    Ok(())
}

// ==================================================
// 3. JSON PAYLOAD PERFORMANCE (plots/json_payload_performance.png)
// ==================================================
pub fn generate_json_chart(json_results: &[(&str, usize, f64, f64)]) -> Result<(), String> {
    ensure_plots_dir()?;
    let path = "plots/json_payload_performance.png";
    let root = BitMapBackend::new(path, (950, 600)).into_drawing_area();
    root.fill(&BG_COLOR).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("🧬 JSON CACHING PERFORMANCE BY PAYLOAD SIZE", ("sans-serif", 24, &TEXT_COLOR))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            -0.5..3.5,
            0.0..6_000_000.0,
        )
        .unwrap();

    chart.configure_mesh()
        .light_line_style(GRID_COLOR)
        .bold_line_style(GRID_COLOR)
        .axis_style(GRID_COLOR)
        .y_desc("Read Operations / Sec")
        .y_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "1KB JSON".to_string(),
                    1 => "10KB JSON".to_string(),
                    2 => "50KB JSON".to_string(),
                    3 => "100KB JSON".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    let mut r_redis = vec![0.0; 4];
    let mut r_keydb = vec![0.0; 4];
    let mut r_omega = vec![0.0; 4];

    for &(engine, size_kb, _write, read) in json_results {
        let idx = match size_kb {
            1 => 0,
            10 => 1,
            50 => 2,
            100 => 3,
            _ => continue,
        };
        match engine {
            "Redis" => r_redis[idx] = read,
            "KeyDB" => r_keydb[idx] = read,
            "OmegaDrive" => r_omega[idx] = read,
            _ => {}
        }
    }

    // Draw Redis bars (offset: center - 0.25, width: 0.22)
    chart.draw_series(
        r_redis.iter().enumerate().map(|(idx, &val)| {
            let center = idx as f64 - 0.25;
            Rectangle::new(
                [
                    (center - 0.11, 0.0),
                    (center + 0.11, val),
                ],
                REDIS_COLOR.mix(0.85).filled(),
            )
        })
    ).unwrap();

    // Draw KeyDB bars (offset: center, width: 0.22)
    chart.draw_series(
        r_keydb.iter().enumerate().map(|(idx, &val)| {
            let center = idx as f64;
            Rectangle::new(
                [
                    (center - 0.11, 0.0),
                    (center + 0.11, val),
                ],
                KEYDB_COLOR.mix(0.85).filled(),
            )
        })
    ).unwrap();

    // Draw OmegaDrive bars (offset: center + 0.25, width: 0.22)
    chart.draw_series(
        r_omega.iter().enumerate().map(|(idx, &val)| {
            let center = idx as f64 + 0.25;
            Rectangle::new(
                [
                    (center - 0.11, 0.0),
                    (center + 0.11, val),
                ],
                OMEGA_COLOR.mix(0.85).filled(),
            )
        })
    ).unwrap();

    root.present().map_err(|e| format!("Failed to save chart: {}", e))?;
    Ok(())
}

// ==================================================
// 4. FILE CACHING BANDWIDTH (plots/file_caching_speed.png)
// ==================================================
pub fn generate_file_chart(file_results: &[(&str, usize, f64, f64)]) -> Result<(), String> {
    ensure_plots_dir()?;
    let path = "plots/file_caching_speed.png";
    let root = BitMapBackend::new(path, (900, 600)).into_drawing_area();
    root.fill(&BG_COLOR).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("💾 BINARY CACHING BANDWIDTH SPEED (MB/s)", ("sans-serif", 24, &TEXT_COLOR))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            -0.5..2.5,
            0.0..6000.0,
        )
        .unwrap();

    chart.configure_mesh()
        .light_line_style(GRID_COLOR)
        .bold_line_style(GRID_COLOR)
        .axis_style(GRID_COLOR)
        .y_desc("Read Bandwidth (MB/s)")
        .y_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "1MB Chunk".to_string(),
                    1 => "5MB Chunk".to_string(),
                    2 => "10MB Chunk".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    let mut b_redis = vec![0.0; 3];
    let mut b_keydb = vec![0.0; 3];
    let mut b_omega = vec![0.0; 3];

    for &(engine, size_mb, _write, read) in file_results {
        let idx = match size_mb {
            1 => 0,
            5 => 1,
            10 => 2,
            _ => continue,
        };
        match engine {
            "Redis" => b_redis[idx] = read,
            "KeyDB" => b_keydb[idx] = read,
            "OmegaDrive" => b_omega[idx] = read,
            _ => {}
        }
    }

    // Draw Redis bars (offset: center - 0.25, width: 0.22)
    chart.draw_series(
        b_redis.iter().enumerate().map(|(idx, &val)| {
            let center = idx as f64 - 0.25;
            Rectangle::new(
                [
                    (center - 0.11, 0.0),
                    (center + 0.11, val),
                ],
                REDIS_COLOR.mix(0.85).filled(),
            )
        })
    ).unwrap();

    // Draw KeyDB bars (offset: center, width: 0.22)
    chart.draw_series(
        b_keydb.iter().enumerate().map(|(idx, &val)| {
            let center = idx as f64;
            Rectangle::new(
                [
                    (center - 0.11, 0.0),
                    (center + 0.11, val),
                ],
                KEYDB_COLOR.mix(0.85).filled(),
            )
        })
    ).unwrap();

    // Draw OmegaDrive bars (offset: center + 0.25, width: 0.22)
    chart.draw_series(
        b_omega.iter().enumerate().map(|(idx, &val)| {
            let center = idx as f64 + 0.25;
            Rectangle::new(
                [
                    (center - 0.11, 0.0),
                    (center + 0.11, val),
                ],
                OMEGA_COLOR.mix(0.85).filled(),
            )
        })
    ).unwrap();

    root.present().map_err(|e| format!("Failed to save chart: {}", e))?;
    Ok(())
}

// ==================================================
// 5. SECURE DATABASE SHOWDOWN (plots/crypto_performance.png)
// ==================================================
pub fn generate_crypto_chart(crypto_results: &[(&str, f64, f64)]) -> Result<(), String> {
    ensure_plots_dir()?;
    let path = "plots/crypto_performance.png";
    let root = BitMapBackend::new(path, (950, 600)).into_drawing_area();
    root.fill(&BG_COLOR).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("🛡️ SECURE DATABASE SHOWDOWN: NEURAL XOR vs CLIENT-SIDE AES-256-GCM", ("sans-serif", 20, &TEXT_COLOR))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            -0.5..5.5,
            0.0..6_000_000.0,
        )
        .unwrap();

    chart.configure_mesh()
        .light_line_style(GRID_COLOR)
        .bold_line_style(GRID_COLOR)
        .axis_style(GRID_COLOR)
        .y_desc("Operations per Second")
        .y_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_style(("sans-serif", 10, &TEXT_COLOR))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "Redis + AES (W)".to_string(),
                    1 => "Redis + AES (R)".to_string(),
                    2 => "KeyDB + AES (W)".to_string(),
                    3 => "KeyDB + AES (R)".to_string(),
                    4 => "Omega Neural (W)".to_string(),
                    5 => "Omega Neural (R)".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    let mut scores = vec![0.0; 6];
    for &(engine, write, read) in crypto_results {
        match engine {
            "Redis + AES" => {
                scores[0] = write;
                scores[1] = read;
            }
            "KeyDB + AES" => {
                scores[2] = write;
                scores[3] = read;
            }
            "OmegaDrive" => {
                scores[4] = write;
                scores[5] = read;
            }
            _ => {}
        }
    }

    chart.draw_series(
        scores.iter().enumerate().map(|(idx, &val)| {
            let color = match idx {
                0 | 1 => REDIS_COLOR,
                2 | 3 => KEYDB_COLOR,
                _ => OMEGA_COLOR,
            };
            Rectangle::new(
                [
                    (idx as f64 - 0.35, 0.0),
                    (idx as f64 + 0.35, val),
                ],
                color.filled(),
            )
        })
    ).unwrap();

    root.present().map_err(|e| format!("Failed to save chart: {}", e))?;
    Ok(())
}

// ==================================================
// 6. TCP PERFORMANCE CHART (plots/tcp_performance.png)
// ==================================================
pub fn generate_tcp_chart(tcp_results: &[(&str, f64, f64)]) -> Result<(), String> {
    ensure_plots_dir()?;
    let path = "plots/tcp_performance.png";
    let root = BitMapBackend::new(path, (900, 600)).into_drawing_area();
    root.fill(&BG_COLOR).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .caption("⚔️ TCP SHOWDOWN: RAW THROUGHPUT (32 CLIENTS)", ("sans-serif", 24, &TEXT_COLOR))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            -0.5..5.5,
            0.0..10_000_000.0,
        )
        .unwrap();

    chart.configure_mesh()
        .light_line_style(GRID_COLOR)
        .bold_line_style(GRID_COLOR)
        .axis_style(GRID_COLOR)
        .y_desc("Operations per Second")
        .y_label_style(("sans-serif", 12, &TEXT_COLOR))
        .x_label_style(("sans-serif", 10, &TEXT_COLOR))
        .x_label_formatter(&|x: &f64| {
            let val = x.round() as i32;
            if (x - val as f64).abs() < 0.01 {
                match val {
                    0 => "Redis-W".to_string(),
                    1 => "Redis-R".to_string(),
                    2 => "KeyDB-W".to_string(),
                    3 => "KeyDB-R".to_string(),
                    4 => "OmegaDrive-W".to_string(),
                    5 => "OmegaDrive-R".to_string(),
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            }
        })
        .draw()
        .unwrap();

    // Map engines to scores
    let mut scores = vec![0.0; 6];
    for &(engine, write, read) in tcp_results {
        match engine {
            "Redis" => {
                scores[0] = write;
                scores[1] = read;
            }
            "KeyDB" => {
                scores[2] = write;
                scores[3] = read;
            }
            "OmegaDrive" => {
                scores[4] = write;
                scores[5] = read;
            }
            _ => {}
        }
    }

    // Draw bars centered at each index with a width of 0.7
    chart.draw_series(
        scores.iter().enumerate().map(|(idx, &val)| {
            let color = match idx {
                0 | 1 => REDIS_COLOR,
                2 | 3 => KEYDB_COLOR,
                _ => OMEGA_COLOR,
            };
            Rectangle::new(
                [
                    (idx as f64 - 0.35, 0.0),
                    (idx as f64 + 0.35, val),
                ],
                color.filled(),
            )
        })
    ).unwrap();

    root.present().map_err(|e| format!("Failed to save chart: {}", e))?;
    Ok(())
}


