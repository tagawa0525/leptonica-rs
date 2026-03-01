/// Compare C and Rust golden images pixel-by-pixel.
///
/// Reads a TSV mapping file that defines C↔Rust golden correspondences,
/// loads both images via the leptonica crate, and reports per-pixel differences.
///
/// # Prerequisites
///
/// 1. Build C leptonica test programs:
///    ```
///    cd reference/leptonica
///    nix develop --command bash -c "rm -rf build && mkdir build && cd build && \
///      cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_PROG=ON && cmake --build . -j\$(nproc)"
///    ```
///
/// 2. Generate C golden files:
///    ```
///    cd reference/leptonica
///    nix develop --command bash -c "cd prog && ../build/bin/<test>_reg generate"
///    ```
///
/// 3. Generate Rust golden files:
///    ```
///    REGTEST_MODE=generate cargo test --test <module>
///    ```
///
/// # Usage
///
/// ```
/// cargo run --example compare_golden --features all-formats -- [OPTIONS]
///
/// Options:
///   --c-dir <DIR>       C regout directory (default: /tmp/lept/regout)
///   --rust-dir <DIR>    Rust regout directory (default: tests/regout)
///   --map <FILE>        TSV mapping file (default: scripts/golden_map.tsv)
///   --module <NAME>     Filter by module name (e.g., "edge", "filter")
///   --threshold <PCT>   Max diff% to classify as fp (default: 5.0)
///   --max-channel <N>   Max channel diff to classify as fp (default: 3)
/// ```
///
/// # Mapping file format (TSV)
///
/// ```
/// # module  c_prefix  c_index  rust_prefix  rust_index  description
/// edge      edge      0        edge         1           Sobel horizontal (1bpp inv)
/// ```
///
/// Lines starting with `#` are comments.  Fields are tab-separated.
use leptonica::io::read_image;
use std::path::Path;

struct Config {
    c_dir: String,
    rust_dir: String,
    map_file: String,
    module_filter: Option<String>,
    fp_threshold_pct: f64,
    fp_max_channel: u32,
}

struct Mapping {
    module: String,
    c_prefix: String,
    c_index: u32,
    rust_prefix: String,
    rust_index: u32,
    description: String,
}

#[derive(Clone)]
#[allow(dead_code)]
struct CompareResult {
    key: String,
    status: String,
    diff_pixels: u64,
    max_diff: u32,
    total_pixels: u64,
    pct: f64,
}

fn parse_args() -> Config {
    let mut cfg = Config {
        c_dir: "/tmp/lept/regout".to_string(),
        rust_dir: format!("{}/tests/regout", env!("CARGO_MANIFEST_DIR")),
        map_file: format!("{}/scripts/golden_map.tsv", env!("CARGO_MANIFEST_DIR")),
        module_filter: None,
        fp_threshold_pct: 5.0,
        fp_max_channel: 3,
    };

    let usage = "Usage: compare_golden [--c-dir DIR] [--rust-dir DIR] [--map FILE] [--module NAME] [--threshold PCT] [--max-channel N]";
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let flag = args[i].as_str();
        let needs_value = matches!(
            flag,
            "--c-dir" | "--rust-dir" | "--map" | "--module" | "--threshold" | "--max-channel"
        );
        if needs_value && i + 1 >= args.len() {
            eprintln!("Missing value for {}\n{}", flag, usage);
            std::process::exit(1);
        }
        match flag {
            "--c-dir" => {
                i += 1;
                cfg.c_dir = args[i].clone();
            }
            "--rust-dir" => {
                i += 1;
                cfg.rust_dir = args[i].clone();
            }
            "--map" => {
                i += 1;
                cfg.map_file = args[i].clone();
            }
            "--module" => {
                i += 1;
                cfg.module_filter = Some(args[i].clone());
            }
            "--threshold" => {
                i += 1;
                cfg.fp_threshold_pct = args[i].parse().expect("invalid threshold");
            }
            "--max-channel" => {
                i += 1;
                cfg.fp_max_channel = args[i].parse().expect("invalid max-channel");
            }
            "--help" | "-h" => {
                eprintln!(
                    "Usage: compare_golden [--c-dir DIR] [--rust-dir DIR] [--map FILE] [--module NAME] [--threshold PCT] [--max-channel N]"
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    cfg
}

fn load_mappings(path: &str, module_filter: Option<&str>) -> Vec<Mapping> {
    let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error reading mapping file {}: {}", path, e);
        std::process::exit(1);
    });

    let mut mappings = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 6 {
            eprintln!("Warning: skipping malformed line: {}", line);
            continue;
        }
        let module = fields[0].trim().to_string();
        if let Some(filter) = module_filter {
            if module != filter {
                continue;
            }
        }
        mappings.push(Mapping {
            module,
            c_prefix: fields[1].trim().to_string(),
            c_index: fields[2].trim().parse().unwrap_or_else(|_| {
                eprintln!("Warning: invalid c_index in: {}", line);
                0
            }),
            rust_prefix: fields[3].trim().to_string(),
            rust_index: fields[4].trim().parse().unwrap_or_else(|_| {
                eprintln!("Warning: invalid rust_index in: {}", line);
                0
            }),
            description: fields[5].trim().to_string(),
        });
    }
    mappings
}

fn find_file(dir: &str, prefix: &str, idx: u32) -> Option<(String, String)> {
    for ext in &["png", "jpg", "tif", "bmp"] {
        let path = format!("{}/{}.{:02}.{}", dir, prefix, idx, ext);
        if Path::new(&path).exists() {
            return Some((path, ext.to_string()));
        }
    }
    None
}

fn pixel_max_channel_diff(a: u32, b: u32, depth: u32) -> u32 {
    if depth <= 8 {
        (a as i32 - b as i32).unsigned_abs()
    } else {
        // 32bpp RGBA: compare each byte channel
        let channels = [24, 16, 8, 0];
        channels
            .iter()
            .map(|shift| {
                let ca = ((a >> shift) & 0xFF) as i32;
                let cb = ((b >> shift) & 0xFF) as i32;
                (ca - cb).unsigned_abs()
            })
            .max()
            .unwrap_or(0)
    }
}

fn compare_one(c_path: &str, r_path: &str) -> Result<(u64, u32, u64), String> {
    let c_pix = read_image(c_path).map_err(|e| format!("C load: {}", e))?;
    let r_pix = read_image(r_path).map_err(|e| format!("R load: {}", e))?;

    if c_pix.width() != r_pix.width() || c_pix.height() != r_pix.height() {
        return Err(format!(
            "DIM C={}x{} R={}x{}",
            c_pix.width(),
            c_pix.height(),
            r_pix.width(),
            r_pix.height()
        ));
    }

    if c_pix.depth().bits() != r_pix.depth().bits() {
        return Err(format!(
            "DEPTH C={} R={}",
            c_pix.depth().bits(),
            r_pix.depth().bits()
        ));
    }

    let w = c_pix.width();
    let h = c_pix.height();
    let total = w as u64 * h as u64;
    let mut diff_count: u64 = 0;
    let mut max_diff: u32 = 0;

    for y in 0..h {
        for x in 0..w {
            let cv = c_pix.get_pixel_unchecked(x, y);
            let rv = r_pix.get_pixel_unchecked(x, y);
            if cv != rv {
                diff_count += 1;
                let d = pixel_max_channel_diff(cv, rv, c_pix.depth().bits());
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
    }

    Ok((diff_count, max_diff, total))
}

fn main() {
    let cfg = parse_args();
    let mappings = load_mappings(&cfg.map_file, cfg.module_filter.as_deref());

    if mappings.is_empty() {
        eprintln!("No mappings found. Check --map file and --module filter.");
        std::process::exit(1);
    }

    println!("\n=== C vs Rust Golden Pixel Comparison ===");
    println!("C dir:    {}", cfg.c_dir);
    println!("Rust dir: {}", cfg.rust_dir);
    if let Some(ref m) = cfg.module_filter {
        println!("Module:   {}", m);
    }
    println!();
    println!(
        "{:<55} {:>12} {:>8} {:>8} {}",
        "Comparison", "Diff/Total", "MaxDiff", "%Diff", "Status"
    );
    println!("{}", "─".repeat(100));

    let mut results: Vec<CompareResult> = Vec::new();

    for m in &mappings {
        let key = format!(
            "{}/{}[C:{:02}↔R:{:02}] {}",
            m.module, m.c_prefix, m.c_index, m.rust_index, m.description
        );

        let c_file = find_file(&cfg.c_dir, &m.c_prefix, m.c_index);
        let r_file = find_file(&cfg.rust_dir, &m.rust_prefix, m.rust_index);

        match (c_file, r_file) {
            (None, _) => {
                println!("{:<55} C NOT FOUND", key);
                results.push(CompareResult {
                    key,
                    status: "C_MISSING".into(),
                    diff_pixels: 0,
                    max_diff: 0,
                    total_pixels: 0,
                    pct: 0.0,
                });
            }
            (_, None) => {
                println!("{:<55} R NOT FOUND", key);
                results.push(CompareResult {
                    key,
                    status: "R_MISSING".into(),
                    diff_pixels: 0,
                    max_diff: 0,
                    total_pixels: 0,
                    pct: 0.0,
                });
            }
            (Some((cp, _)), Some((rp, _))) => match compare_one(&cp, &rp) {
                Err(msg) => {
                    println!("{:<55} {}", key, msg);
                    results.push(CompareResult {
                        key,
                        status: "DIM_MISMATCH".into(),
                        diff_pixels: 0,
                        max_diff: 0,
                        total_pixels: 0,
                        pct: 0.0,
                    });
                }
                Ok((diff, max_d, total)) => {
                    let pct = if total > 0 {
                        diff as f64 / total as f64 * 100.0
                    } else {
                        0.0
                    };
                    let status = if diff == 0 {
                        "IDENTICAL"
                    } else if pct < cfg.fp_threshold_pct && max_d <= cfg.fp_max_channel {
                        "diff(fp)"
                    } else {
                        "DIFF(alg)"
                    };

                    println!(
                        "{:<55} {:>6}/{:<6} {:>8} {:>7.2}% {}",
                        key, diff, total, max_d, pct, status
                    );
                    results.push(CompareResult {
                        key,
                        status: status.into(),
                        diff_pixels: diff,
                        max_diff: max_d,
                        total_pixels: total,
                        pct,
                    });
                }
            },
        }
    }

    // Summary
    let count = |s: &str| results.iter().filter(|r| r.status == s).count();
    let missing = results
        .iter()
        .filter(|r| r.status.contains("MISSING"))
        .count();

    println!("\n=== Summary ===");
    println!("Identical:          {}", count("IDENTICAL"));
    println!("Float diff:         {}", count("diff(fp)"));
    println!("Algorithm diff:     {}", count("DIFF(alg)"));
    println!("Dimension mismatch: {}", count("DIM_MISMATCH"));
    println!("Missing:            {}", missing);
    println!("Total:              {}", results.len());

    if count("DIFF(alg)") > 0 {
        std::process::exit(1);
    }
}
