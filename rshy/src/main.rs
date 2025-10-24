use std::process::Command;
use std::env;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::{self, Write, Read};
use std::process::Stdio;
use std::time::Duration;
use sha2::{Sha256, Digest};
use rand::Rng;
use std::thread;
use serde_json::Value;
use tokio;
use tokio::time::timeout;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use num_cpus;
use regex;
use indicatif::{ProgressBar, ProgressStyle};
use futures::StreamExt;
use rusqlite::{Connection, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use zip::ZipWriter;
use zip::write::FileOptions;
use std::os::unix::fs::PermissionsExt;

type AppResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> AppResult {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Err("参数不足".into());
    }

    match handle_command(&args).await {
        Ok(()) => {
            std::process::exit(0);
        },
        Err(_) => {
            std::process::exit(1);
        }
    }
}

async fn handle_command(args: &[String]) -> AppResult {
    match args[1].as_str() {
        "-V" | "--version" => show_version(),
        "-h" | "--help" => {
            print_help();
            Ok(())
        },
        "-o" | "--download" => handle_download(args).await,
        "-d" | "--delete" => handle_delete(args),
        "-t" | "--tools" => handle_tools(args),
        "-i" | "--integritycheck" => handle_integrity_check(args),
        "--color" => {
            show_startup_animation();
            Ok(())
        },
        "--checkzygisk" => check_zygisk(),
        "--yiyan" => show_yiyan().await,
        "--update" => handle_update().await,
        "--cleanmodules" => handle_clean_modules(),
        "initrc" => init_rc(),
        "hidemyapplist" => hidemyapplist().await,
        "recoverapplist" => recoverapplist(),
        "lsplog" => {
            clean_lsplog();
            Ok(())
        },
        "magisklog" => handle_magisklog(),
        "shamiko_pattern" => {
            shamiko_modules();
            Ok(())
        },
        "cts" => {
            cts_fix();
            Ok(())
        },
        "updatetarget" => update_target_file(),
        "awjclean" => handle_awjclean(),
        "aptroot" => handle_aptroot(),
        "rurudelete" => handle_rurudelete(),
        "nativetest" => handle_nativetest(args).await,
        "holmes" => handle_holmes(args),
        "momo" => handle_momo(args),
        "hunter" => handle_hunter(args),
        "nativedetector" => handle_nativedetector(args).await,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_help();
            std::process::exit(1);
        }
    }
}

fn handle_clean_modules() -> AppResult {
    clean_modules_dirs()?;
    Ok(())
}

fn show_version() -> AppResult {
    println!("v5.0.5");
    Ok(())
}

fn handle_magisklog() -> AppResult {
    let _ = deleter("file", "/cache/magisk.log", false);
    let _ = deleter("file", "/cache/magisk.log.bak", false);
    Ok(())
}

fn handle_awjclean() -> AppResult {
    let _ = deleter("dir", "/storage/emulated/0/Android/data/com.byyoung.setting", false);
    let _ = deleter("dir", "/storage/emulated/0/Android/obb/com.byyoung.setting", false);
    Ok(())
}

fn handle_aptroot() -> AppResult {
    let _ = deleter("all", "/data/local/tmp", true);
    let _ = deleter("dir", "/data/core", false);
    let _ = deleter("dir", "/data/duraspeed", false);
    let _ = deleter("dir", "/data/dumpsys", false);
    let _ = deleter("file", "/data/swap_config.conf", false);
    Ok(())
}

fn handle_rurudelete() -> AppResult {
    let _ = deleter("dir", "/data/xedge", false);
    let _ = deleter("dir", "/data/xlua", false);
    let _ = deleter("dir", "/sdcard/TWRP", false);
    Ok(())
}

fn handle_delete(args: &[String]) -> AppResult {
    if args.len() < 4 {
        return Err("Delete type and path are required".into());
    }

    let recursive = args.len() > 4 && args[4] == "-r";
    let _ = deleter(&args[2], &args[3], recursive);
    Ok(())
}

async fn handle_nativetest(args: &[String]) -> AppResult {
    if args.len() < 3 {
        print_help();
        return Err("参数不足".into());
    }
    nativetest(&args[2]).await
}

fn handle_holmes(args: &[String]) -> AppResult {
    if args.len() < 3 {
        print_help();
        return Err("参数不足".into());
    }
    holmes(&args[2])
}

fn handle_momo(args: &[String]) -> AppResult {
    if args.len() < 3 {
        print_help();
        return Err("参数不足".into());
    }
    momo(&args[2])
}

fn handle_hunter(args: &[String]) -> AppResult {
    if args.len() < 3 {
        print_help();
        return Err("参数不足".into());
    }
    hunter(&args[2])
}

async fn handle_nativedetector(args: &[String]) -> AppResult {
    if args.len() < 3 {
        print_help();
        return Err("参数不足".into());
    }
    nativedetector(&args[2]).await
}

async fn handle_download(args: &[String]) -> AppResult {
    if args.len() < 3 {
        return Err("Download URL is required".into());
    }

    let url = &args[2];
    let mut use_cdn = true;
    let mut save_path = None;
    let mut expected_hash = None;
    
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--no-cdn" => {
                use_cdn = false;
                i += 1;
            }
            _ if save_path.is_none() => {
                save_path = Some(std::path::PathBuf::from(&args[i]));
                i += 1;
            }
            _ if expected_hash.is_none() => {
                expected_hash = Some(args[i].clone());
                i += 1;
            }
            _ => {
                eprintln!("未知参数: {}", args[i]);
                print_help();
                return Err("未知参数".into());
            }
        }
    }

    download_file(url.clone(), use_cdn, save_path, expected_hash).await?;
    Ok(())
}

fn handle_tools(args: &[String]) -> AppResult {
    if args.len() < 3 {
        return Err("Tool name is required".into());
    }
    let tool_name = &args[2];
    run_useful_tool(tool_name)?;
    println!("工具执行完成");
    Ok(())
}

fn handle_integrity_check(args: &[String]) -> AppResult {
    if args.len() < 4 {
        return Err("File path and expected hash value are required".into());
    }
    let file_path = &args[2];
    let expected_hash = &args[3];
    integrity_check(file_path, expected_hash)
}

fn print_help() {
    eprintln!("rshy - EHoRY Cli");
    eprintln!();
    eprintln!("Usage: rshy <Arguments> <Extra arguments>");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("  hidemyapplist");
    eprintln!("  recoverapplist");
    eprintln!("  lsplog");
    eprintln!("  magisklog");
    eprintln!("  shamiko_pattern");
    eprintln!("  updatetarget");
    eprintln!("  awjclean");
    eprintln!("  rurudelete");
    eprintln!("  aptroot");
    eprintln!("  initrc");
    eprintln!("  momo [Extra <tee> / <systemmount> / <development> / <sdk> / <addon> ]");
    eprintln!("  nativetest [Extra <futile10> / <boothash> ]");
    eprintln!("  holmes [Extra <somethingwrong> / <9ff> ]");
    eprintln!("  hunter [Extra <shizuku> / <manager> ]");
    eprintln!("  nativedetector [Extra <vbmeta> / <magicmount> / <lsp5> ]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -h, --help");
    eprintln!("  -V, --version");
    eprintln!("  -i, --integritycheck <file_path> <expected_hash>");
    eprintln!("  -d, --delete <file|dir|files_in_dir> <path>");
    eprintln!("  -o, --download <URL> [save_path] [expected_hash] [--no-cdn]");
    eprintln!("  -t, --tools <tool_name>");
    eprintln!("  --color");
    eprintln!("  --yiyan");
    eprintln!("  --zygiskcheck");
    eprintln!("  --update");
    eprintln!("  --cleanmodules");
}

fn compute_sha256(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

async fn handle_update() -> AppResult {
    let url = "https://textdb.online/y13140".to_string();
    let temp_file = std::env::temp_dir().join("y13140");
    download_small_file_silent(&url, &temp_file).await?;
    let content = fs::read_to_string(&temp_file)?;
    let _ = fs::remove_file(&temp_file);

    let version = content.trim().replace(|c: char| c.is_whitespace(), "");
    println!("{}", version);
    
    Ok(())
}

async fn download_small_file_silent(url: &str, file_path: &std::path::Path) -> AppResult {
    let client = Client::new();

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP错误: {}", response.status()).into());
    }

    let content = response.bytes()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    let mut file = File::create(file_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    
    file.write_all(&content)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(())
}

fn integrity_check(file_path: &str, expected_hash: &str) -> AppResult {
    if !Path::new(file_path).exists() {
        return Err("未检测到文件，请确认是否下载完毕".into());
    }

    let computed_hash = compute_sha256(file_path)?;
    if computed_hash == expected_hash {
        println!("sha256完整性校验通过");
        Ok(())
    } else {
        eprintln!("sha256完整性校验未通过");
        eprintln!("请检查文件是否已经下载100%");
        eprintln!("如果未发现其他问题，请私信@yu13140报告此错误");
        std::process::exit(1);
    }
}

fn check_zygisk() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = "/data/adb/magisk.db";

    if !Path::new(db_path).exists() {
        println!("Magisk 数据库文件未找到");
        return Ok(());
    }

    let conn = match Connection::open(db_path) {
        Ok(conn) => conn,
        Err(e) => {
            println!("无法打开 Magisk 数据库: {}", e);
            return Ok(());
        }
    };

    let mut stmt = match conn.prepare("SELECT value FROM settings WHERE key='zygisk'") {
        Ok(stmt) => stmt,
        Err(e) => {
            println!("无法准备 SQL 查询: {}", e);
            return Ok(());
        }
    };
    
    let current_state: Result<i32, rusqlite::Error> = stmt.query_row([], |row| row.get(0));
    
    match current_state {
        Ok(state) => {
            println!("{}", state);
        },
        Err(e) => {
            println!("无法查询 Zygisk 状态: {}", e);
        }
    }
    
    Ok(())
}

fn show_startup_animation() {
    print!("\x1B[?25l");
    io::stdout().flush().unwrap();

    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();

    let title_lines = [
        ".########.##.....##..#######..########..##....##...",
        ".##.......##.....##.##.....##.##.....##..##..##....",
        ".##.......##.....##.##.....##.##.....##...####.....",
        ".######...#########.##.....##.########.....##......",
        ".##.......##.....##.##.....##.##...##......##......",
        ".##.......##.....##.##.....##.##....##.....##......",
        ".########.##.....##..#######..##.....##....##......",
    ];

    let colors = [196, 202, 208, 214, 220, 226, 190, 154, 118, 82, 46, 47, 48, 49, 50, 51, 45, 39, 33, 27, 21];
    let progress_width = 20;
    let increment = 2;

    let total_cols = title_lines[0].len();

    println!("\n\n");

    for _ in 0..7 {
        for _ in 0..total_cols {
            print!(" ");
        }
        println!();
    }

    fn print_title_by_column(title_lines: &[&str], percentage: u32, colors: &[u32]) {
        let total_cols = title_lines[0].len();
        let cols_to_show = (total_cols as u32 * percentage / 100) as usize;
        
        let mut output = String::new();
        for col in 0..total_cols {
            if col < cols_to_show {
                for row in 0..7 {
                    let char = title_lines[row].chars().nth(col).unwrap();
                    if char != ' ' {
                        let color_index = (col + row) % colors.len();
                        let color = colors[color_index];
                        output.push_str(&format!("\x1B[{};{}H\x1B[38;5;{}m{}\x1B[0m", row + 3, col + 1, color, char));
                    }
                }
            }
        }
        print!("{}", output);
        io::stdout().flush().unwrap();
    }

    fn color_reverse_effect(title_lines: &[&str], colors: &[u32]) {
        let total_cols = title_lines[0].len();
        let reverse_colors: Vec<u32> = colors.iter().rev().cloned().collect();

        for col in (0..total_cols).rev() {
            let mut output = String::new();
            for row in 0..7 {
                let char = title_lines[row].chars().nth(col).unwrap();
                if char != ' ' {
                    let color_index = (col * reverse_colors.len() / total_cols) % reverse_colors.len();
                    let color = reverse_colors[color_index];
                    output.push_str(&format!("\x1B[{};{}H\x1B[38;5;{}m{}\x1B[0m", row + 3, col + 1, color, char));
                }
            }
            print!("{}", output);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(50));
        }

        for col in (0..total_cols).rev() {
            let mut output = String::new();
            for row in 0..7 {
                let char = title_lines[row].chars().nth(col).unwrap();
                if char != ' ' {
                    let color_index = (col * colors.len() / total_cols) % colors.len();
                    let color = colors[color_index];
                    output.push_str(&format!("\x1B[{};{}H\x1B[38;5;{}m{}\x1B[0m", row + 3, col + 1, color, char));
                }
            }
            print!("{}", output);
            io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn print_progress(percentage: u32, progress_width: usize, colors: &[u32]) {
        let completed = (percentage as usize * progress_width / 100) as usize;
        print!("\r\x1B[1;37mProgress: \x1B[0m[");

        for i in 0..progress_width {
            if i < completed {
                let color_index = rand::thread_rng().gen_range(0..colors.len());
                let color = colors[color_index];
                print!("\x1B[48;5;{};1m \x1B[0m", color);
            } else {
                print!(" ");
            }
        }

        print!("] \x1B[1;32m{}%\x1B[0m", percentage);
        io::stdout().flush().unwrap();
    }

    let mut percentage = 0;
    while percentage <= 100 {
        print_title_by_column(&title_lines, percentage, &colors);
        print!("\x1B[11;1H");
        print_progress(percentage, progress_width, &colors);
        thread::sleep(Duration::from_millis(50));
        percentage += increment;
    }
    println!();

    println!("\n\x1B[1;32m✓ SYSTEM READY\x1B[0m");
    thread::sleep(Duration::from_millis(500));

    color_reverse_effect(&title_lines, &colors);
    
    thread::sleep(Duration::from_millis(500));

    println!("\n");
    
    let license_text = "EHoRY © 2025 by yu13140 is licensed under Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International. To view a copy of this license, visit https://creativecommons.org/licenses/by-nc-sa/4.0/";

    let terminal_width = 80;
    let words: Vec<&str> = license_text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    
    for word in words {
        if current_line.len() + word.len() + 1 > terminal_width {
            lines.push(current_line.clone());
            current_line.clear();
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    for (i, line) in lines.iter().enumerate() {
        let start_row = 15 + i;

        print!("\x1B[{};1H\x1B[38;5;15m{}\x1B[0m", start_row, line);
        io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(100));
    }

    println!("\x1B[{};1H", 15 + lines.len() + 1);

    thread::sleep(Duration::from_secs(2));

    print!("\x1B[?25h");
    io::stdout().flush().unwrap();
}

async fn show_yiyan() -> Result<(), Box<dyn std::error::Error>> {
    let api = "https://v1.hitokoto.cn/?";
    let client = Client::new();
    let response = client.get(api).send().await?;
    let json: Value = response.json().await?;
    
    let word = json["hitokoto"].as_str().unwrap_or("");
    let from = json["from"].as_str().unwrap_or("");

    let mut show = word.to_string();
    
    fn is_ok(s: &str) -> bool {
        let no_mean = vec!["null", "NULL", "自己", "原创", "其它", "其他", "网络", "来自网络"];
        !no_mean.contains(&s)
    }
    
    if is_ok(from) {
        show.push_str(&format!("\n「 {} 」", from));
    }
    
    println!("{}", show);
    Ok(())
}

#[derive(Debug, Clone)]
struct CdnNode {
    name: String,
    url: String,
}

fn get_cdn_nodes() -> Vec<CdnNode> {
    vec![
        CdnNode {
            name: "CDN 节点 1".to_string(),
            url: "https://gh.llkk.cc/".to_string(),
        },
//        不可用节点
//        CdnNode {
//            name: "CDN 节点 2".to_string(),
//            url: "https://ghfile.geekertao.top/".to_string(),
//        },
//        CdnNode {
//            name: "CDN 节点 4".to_string(),
//            url: "https://ghproxy.net/".to_string(),
//        },
        CdnNode {
            name: "CDN 节点 2".to_string(),
            url: "https://hk.gh-proxy.com/".to_string(),
        },
//        CdnNode {
//            name: "CDN 节点 3".to_string(),
//            url: "https://github.moeyy.xyz/".to_string(),
//        },
        CdnNode {
            name: "CDN 节点 3".to_string(),
            url: "https://cdn.gh-proxy.com/".to_string(),
        },
        CdnNode {
            name: "CDN 节点 4".to_string(),
            url: "https://edgeone.gh-proxy.com/".to_string(),
        },
        CdnNode {
            name: "CDN 节点 5".to_string(),
            url: "https://gh-proxy.com/".to_string(),
        },
        CdnNode {
            name: "CDN 节点 6".to_string(),
            url: "https://ghf.xn--eqrr82bzpe.top/".to_string(),
        },
    ]
}

async fn test_cdn_speed(nodes: Vec<CdnNode>) -> Vec<DownloadResult> {
    println!("开始测试 CDN 节点速度...");
    
    let mut results = Vec::new();
    let client = Client::new();

    let test_file_url = "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/check.sh";
    let expected_size = 99461;
    
    for node in nodes {
        let cdn_url = format!("{}{}", node.url, test_file_url);

        let test_future = test_single_node(&client, &cdn_url, &node.name, expected_size);
        match timeout(Duration::from_secs(5), test_future).await {
            Ok(Ok(result)) => {
                let display_latency = if result.latency == u128::MAX {
                    "失败".to_string()
                } else {
                    format!("{}ms", result.latency)
                };
                println!("节点 {}: 延迟 {}, 速度 {:.2}MB/s", node.name, display_latency, result.speed);

                results.push(result);
            },
            Ok(Err(e)) => {
                println!("节点 {}: 测试失败 - {}", node.name, e);
                results.push(DownloadResult {
                    node: node.clone(),
                    speed: 0.0,
                    latency: u128::MAX,
                    success: false,
                });
            },
            Err(_) => {
                println!("节点 {}: 测试超时", node.name);
                results.push(DownloadResult {
                    node: node.clone(),
                    speed: 0.0,
                    latency: u128::MAX,
                    success: false,
                });
            }
        }
    }

    results.sort_by(|a, b| {
        if a.success && !b.success {
            std::cmp::Ordering::Less
        } else if !a.success && b.success {
            std::cmp::Ordering::Greater
        } else {
            b.speed.partial_cmp(&a.speed)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.latency.cmp(&b.latency))
        }
    });

    results
}

async fn test_single_node(
    client: &Client,
    cdn_url: &str,
    node_name: &str,
    expected_size: u64,
) -> Result<DownloadResult, Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();

    let latency = match client.head(cdn_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                start_time.elapsed().as_millis()
            } else {
                u128::MAX
            }
        },
        Err(e) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HEAD 请求失败: {}", e),
            )));
        }
    };

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("cdn_speed_test.tmp");

    let speed_result = download_with_progress(client, cdn_url, Some(temp_file.clone()), None, true).await;
    
    let speed = match speed_result {
        Ok((file_path, speed_value)) => {
            match fs::metadata(&file_path) {
                Ok(metadata) => {
                    if metadata.len() == expected_size {
                        speed_value
                    } else {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("文件大小不正确: 期望 {} 字节, 实际 {} 字节", expected_size, metadata.len()),
                        )));
                    }
                },
                Err(e) => {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("无法获取文件信息: {}", e),
                    )));
                }
            }
        },
        Err(e) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("下载测试失败: {}", e),
            )));
        }
    };

    let _ = fs::remove_file(temp_file);
    
    Ok(DownloadResult {
        node: CdnNode {
            name: node_name.to_string(),
            url: cdn_url.to_string(),
        },
        speed,
        latency,
        success: latency != u128::MAX && speed > 0.0,
    })
}

async fn download_with_progress(
    client: &Client,
    url: &str,
    save_path: Option<std::path::PathBuf>,
    max_size: Option<usize>,
    silent: bool,
) -> Result<(String, f64), Box<dyn std::error::Error>> {
    let file_path = save_path.unwrap_or_else(|| {
        let file_name = url.split('/').last().unwrap_or("download");
        std::path::PathBuf::from(file_name)
    });

    let mut downloaded: u64 = 0;
    let mut file = if file_path.exists() {
        let metadata = fs::metadata(&file_path)?;
        downloaded = metadata.len();
        println!("发现已下载部分文件: {} bytes", downloaded);
        std::fs::OpenOptions::new().append(true).open(&file_path)?
    } else {
        File::create(&file_path)?
    };

    let mut request = client.get(url);
    if downloaded > 0 {
        request = request.header("Range", format!("bytes={}-", downloaded));
        println!("尝试断点续传，从字节 {} 开始", downloaded);
    }

    let res = match request.send().await {
        Ok(res) => res,
        Err(e) => {
            eprintln!("请求失败: {}", e);
            return Err(Box::new(e));
        }
    };

    let (total_size, supports_resume) = if downloaded > 0 && res.status() == 206 {
        println!("服务器支持断点续传 (状态码 206)");
        let content_range = res
            .headers()
            .get(reqwest::header::CONTENT_RANGE)
            .and_then(|ct_range| ct_range.to_str().ok());
        
        let total = if let Some(range) = content_range {
            match range.split('/').last().and_then(|s| s.parse::<u64>().ok()) {
                Some(t) => {
                    println!("从Content-Range获取总大小: {}", t);
                    t
                }
                None => {
                    println!("无法从Content-Range解析总大小");
                    res.headers()
                        .get(reqwest::header::CONTENT_LENGTH)
                        .and_then(|ct_len| ct_len.to_str().ok())
                        .and_then(|ct_len| ct_len.parse::<u64>().ok())
                        .unwrap_or(0) + downloaded
                }
            }
        } else {
            println!("没有Content-Range头，使用Content-Length");
            res.headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse::<u64>().ok())
                .unwrap_or(0) + downloaded
        };
        
        (total, true)
    } else if downloaded > 0 {
        println!("服务器不支持断点续传 (状态码: {}), 重新开始下载", res.status());
        downloaded = 0;
        file = File::create(&file_path)?;

        let res = client.get(url).send().await?;
        let total = res
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse::<u64>().ok())
            .unwrap_or(0);
        
        (total, false)
    } else {
        let total = res
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse::<u64>().ok())
            .unwrap_or(0);
        
        (total, false)
    };

    let pb = if !silent {
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"));
        pb.set_position(downloaded);
        Some(pb)
    } else {
        None
    };

    let mut stream = res.bytes_stream();
    let start_time = std::time::Instant::now();
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 10;
    let mut last_speed_check = std::time::Instant::now();
    let mut downloaded_since_last_check = 0;
    let mut estimated_total_time = 0.0;

    while retry_count < MAX_RETRIES {
        let mut attempt_failed = false;
        
        while let Some(item) = stream.next().await {
            match item {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk) {
                        eprintln!("写入文件失败: {}", e);
                        attempt_failed = true;
                        break;
                    }
                    
                    let new = downloaded + (chunk.len() as u64);
                    downloaded = new;
                    downloaded_since_last_check += chunk.len() as u64;

                    if let Some(ref pb) = pb {
                        pb.set_position(downloaded);
                    }

                    let now = std::time::Instant::now();
                    if now.duration_since(last_speed_check).as_secs() >= 5 {
                        let time_elapsed = now.duration_since(last_speed_check).as_secs_f64();
                        let current_speed = downloaded_since_last_check as f64 / time_elapsed;
                        
                        if current_speed > 0.0 && total_size > downloaded {
                            let remaining_bytes = total_size - downloaded;
                            estimated_total_time = remaining_bytes as f64 / current_speed;
                            
                            if let Some(ref pb) = pb {
                                pb.set_message(format!("速度: {:.2} KB/s, 剩余: {:.0}秒", 
                                    current_speed / 1024.0, estimated_total_time));
                            }

                            if estimated_total_time > 600.0 {
                                if let Some(ref pb) = pb {
                                    pb.suspend(|| {
                                        println!("预计下载时间超过10分钟，将重新测试CDN节点");
                                    });
                                } else {
                                    println!("预计下载时间超过10分钟，将重新测试CDN节点");
                                }
                                attempt_failed = true;
                                break;
                            }
                        }
                        
                        last_speed_check = now;
                        downloaded_since_last_check = 0;
                    }

                    if let Some(max) = max_size {
                        if downloaded >= max as u64 {
                            break;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("下载数据出错: {}", e);
                    attempt_failed = true;
                    break;
                }
            }
        }

        if downloaded >= total_size {
            break;
        }
        
        if !attempt_failed {
            break;
        }

        if estimated_total_time > 600.0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "下载速度过慢，预计完成时间超过10分钟",
            )));
        }
        
        retry_count += 1;
        if let Some(ref pb) = pb {
            pb.suspend(|| {
                println!("网络连接出现问题，正在尝试重新连接... (尝试 {}/{})", retry_count, MAX_RETRIES);
            });
            pb.set_message("重新连接中...");
        } else {
            println!("网络连接出现问题，正在尝试重新连接... (尝试 {}/{})", retry_count, MAX_RETRIES);
        }

        tokio::time::sleep(Duration::from_secs((2 * retry_count) as u64)).await;

        file = std::fs::OpenOptions::new().append(true).open(&file_path)?;

        let current_size = fs::metadata(&file_path)?.len();
        downloaded = current_size;

        let mut request = client.get(url);
        if supports_resume {
            request = request.header("Range", format!("bytes={}-", downloaded));
            if let Some(ref pb) = pb {
                pb.suspend(|| {
                    println!("尝试从字节 {} 继续下载", downloaded);
                });
            } else {
                println!("尝试从字节 {} 继续下载", downloaded);
            }
        } else {
            if let Some(ref pb) = pb {
                pb.suspend(|| {
                    println!("服务器不支持断点续传，重新开始下载");
                });
            } else {
                println!("服务器不支持断点续传，重新开始下载");
            }
            downloaded = 0;
            file = File::create(&file_path)?;
        }
        
        match request.send().await {
            Ok(new_res) => {
                stream = new_res.bytes_stream();
                
                if let Some(ref pb) = pb {
                    pb.set_message("下载中");
                    pb.set_position(downloaded);
                }
            }
            Err(e) => {
                eprintln!("重新连接失败: {}", e);
                if retry_count == MAX_RETRIES {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        "多次重试后仍无法恢复下载",
                    )));
                }
            }
        }
    }

    if retry_count == MAX_RETRIES {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            "多次重试后仍无法完成下载",
        )));
    }

    if let Some(pb) = pb {
        pb.finish_with_message("下载完成");
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        (downloaded as f64 / 1024.0 / 1024.0) / elapsed
    } else {
        0.0
    };
    
    if !silent {
        println!("下载速度: {:.2}MB/s", speed);
    }
    
    Ok((file_path.to_string_lossy().into_owned(), speed))
}

async fn download_file(
    url: String,
    mut use_cdn: bool,
    save_path: Option<std::path::PathBuf>,
    expected_hash: Option<String>,
) -> AppResult<String> {
    let vpn_detected = is_vpn_active();
    
    if vpn_detected {
        println!("检测到VPN可能已被开启，是否继续使用CDN加速？(y/N): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().eq_ignore_ascii_case("y") {
            println!("将继续使用CDN加速");
        } else {
            println!("将不使用CDN加速");
            use_cdn = false;
        }
    }

    let client = Client::new();
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 5;

    if !use_cdn {
        println!("不使用 CDN 加速，直接下载");
        let (file_path, _) = timeout(
            Duration::from_secs(600),
            download_with_progress(&client, &url, save_path.clone(), None, false)
        ).await??;
        
        if let Some(expected_hash) = expected_hash {
            integrity_check(&file_path, &expected_hash)?;
        }
        return Ok(file_path);
    }

    let cdn_results = test_cdn_speed(get_cdn_nodes()).await;
    let mut cdn_nodes: Vec<CdnNode> = cdn_results.iter()
        .filter(|r| r.success)
        .map(|r| r.node.clone())
        .collect();
    
    cdn_nodes.push(CdnNode {
        name: "原始URL".to_string(),
        url: "".to_string(),
    });

    for (i, node) in cdn_nodes.iter().enumerate() {
        if retry_count >= MAX_RETRIES {
            break;
        }

        println!("开始下载文件... (尝试 {}/{}, 使用节点: {})", 
                 retry_count + 1, MAX_RETRIES, node.name);

        let final_url = if node.url.is_empty() {
            url.clone()
        } else {
            let cdn_base_url = node.url
                .replace("https://github.com/yu13140/yuhideroot/raw/refs/heads/main/check.sh", "");
            format!("{}{}", cdn_base_url, url)
        };

        match timeout(
            Duration::from_secs(600),
            download_with_progress(&client, &final_url, save_path.clone(), None, false)
        ).await {
            Ok(Ok((file_path, _))) => {
                if let Some(expected_hash) = &expected_hash {
                    integrity_check(&file_path, expected_hash)?;
                }
                return Ok(file_path);
            },
            Ok(Err(e)) => {
                println!("下载失败: {}", e);
                if i < cdn_nodes.len() - 1 {
                    println!("CDN 下载失败，尝试使用下一个节点");
                    continue;
                } else {
                    retry_count += 1;
                }
            },
            Err(_) => {
                println!("下载超时，正在重试...");
                retry_count += 1;
                
                if retry_count < MAX_RETRIES {
                    let wait_time = 5 * retry_count;
                    println!("等待 {} 秒后重试...", wait_time);
                    tokio::time::sleep(Duration::from_secs(wait_time as u64)).await;
                }
            }
        }
    }

    Err(format!("下载失败，经过 {} 次尝试和使用 {} 个节点后仍无法完成", retry_count, cdn_nodes.len()).into())
}

fn is_vpn_active() -> bool {
    println!("正在检查VPN状态...");
    
    let output = Command::new("sh")
        .arg("-c")
        .arg("ip addr | grep -E 'tun[0-9]|ppp[0-9]|wg[0-9]' | head -1")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("检测到VPN网络接口: {}", stdout);
                return true;
            }
        }
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg("ip route | grep -E 'tun|ppp|wg' | head -1")
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("检测到VPN路由: {}", stdout);
                return true;
            }
        }
    }

    let vpn_processes = [
        "openvpn", "wireguard", "wg-quick", "pppd", "strongswan", "ipsec",
        "openconnect", "anyconnect", "forticlient", "nordvpn", "expressvpn",
    ];

    for process in vpn_processes.iter() {
        let output = Command::new("pgrep")
            .arg("-x")
            .arg(process)
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                println!("检测到VPN进程: {}", process);
                return true;
            }
        }
    }

    println!("未检测到VPN活动");
    false
}

#[derive(Debug)]
struct DownloadResult {
    node: CdnNode,
    speed: f64,
    latency: u128,
    success: bool,
}

fn get_all_apk_paths() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = run_useful_tool_with_args("cmd", &[
        "package", "list", "packages", "-f", "-3"
    ])?;
    
    let mut apk_paths = Vec::new();
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if let Some(stripped) = line.strip_prefix("package:") {
                if let Some(equal_pos) = stripped.rfind('=') {
                    let full_apk_path = &stripped[..equal_pos];

                    if full_apk_path.ends_with(".apk") {
                        apk_paths.push(full_apk_path.to_string());
                    } else {
                        let full_path = format!("{}/base.apk", full_apk_path);
                        apk_paths.push(full_path);
                    }
                }
            }
        }
    } else {
        println!("命令失败");
        let error_str = String::from_utf8_lossy(&output.stderr);
        println!("错误信息: {}", error_str);
    }
    
    println!("找到 {} 个 APK 文件路径", apk_paths.len());
    Ok(apk_paths)
}

fn check_apk_for_hma(apk_path: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if !Path::new(apk_path).exists() {
        return Ok(None);
    }

    match fs::metadata(apk_path) {
        Ok(metadata) => {
            if metadata.len() == 0 {
                return Ok(None);
            }
        },
        Err(_) => {
            return Ok(None);
        }
    }

    match File::open(apk_path) {
        Ok(_) => {
            // 文件可访问，继续执行
        },
        Err(_) => {
            return Ok(None);
        }
    }

    let output = match run_useful_tool_with_args("aapt", &[
        "dump",
        "xmltree",
        apk_path,
        "AndroidManifest.xml"
    ]) {
        Ok(output) => output,
        Err(e) => {
            if e.to_string().contains("Text file busy") {
                thread::sleep(Duration::from_millis(100));

                match run_useful_tool_with_args("aapt", &[
                    "dump",
                    "xmltree",
                    apk_path,
                    "AndroidManifest.xml"
                ]) {
                    Ok(output) => output,
                    Err(_) => {
                        return Ok(None);
                    }
                }
            } else {
                return Ok(None);
            }
        }
    };
    
    let output_str = String::from_utf8_lossy(&output.stdout);

    if output_str.contains("icu.nullptr.hidemyapplist") {
        println!("找到疑似隐藏应用列表的 APK: {}", apk_path);

        let package_pattern = regex::Regex::new(r#"package="([^"]*)""#)?;
        if let Some(caps) = package_pattern.captures(&output_str) {
            if let Some(package_name) = caps.get(1) {
                return Ok(Some(package_name.as_str().to_string()));
            }
        }

        for line in output_str.lines() {
            if line.contains("package=") {
                if let Some(start) = line.find("package=") {
                    let rest = &line[start + 8..];
                    if let Some(quote_start) = rest.find('"') {
                        let rest = &rest[quote_start + 1..];
                        if let Some(quote_end) = rest.find('"') {
                            return Ok(Some(rest[..quote_end].to_string()));
                        }
                    }
                }
            }
        }
        
        println!("执行命令成功无法提取包名: {}", apk_path);
    }
    
    Ok(None)
}

fn find_hma_package_with_aapt() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let apk_paths = get_all_apk_paths()?;
    
    println!("找到 {} 个第三方应用APK文件路径", apk_paths.len());
    
    if apk_paths.is_empty() {
        println!("未找到任何第三方应用APK文件");
        return Ok(Vec::new());
    }

    let existing_apk_paths: Vec<String> = apk_paths.into_iter()
        .filter(|path| Path::new(path).exists())
        .collect();
    
    println!("实际可访问的 APK 文件: {} 个", existing_apk_paths.len());
    
    if existing_apk_paths.is_empty() {
        println!("没有可访问的APK文件");
        return Ok(Vec::new());
    }

    let pb = ProgressBar::new(existing_apk_paths.len() as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    let found_packages = Arc::new(Mutex::new(Vec::new()));
    let processed = Arc::new(AtomicU64::new(0));
    
    let chunk_size = (existing_apk_paths.len() / num_cpus::get()).max(1);
    let mut handles = Vec::new();
    
    for chunk in existing_apk_paths.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let found_packages = Arc::clone(&found_packages);
        let processed = Arc::clone(&processed);
        
        handles.push(thread::spawn(move || {
            for apk_path in chunk {
                let mut retries = 0;
                let max_retries = 3;
                
                while retries < max_retries {
                    match check_apk_for_hma(&apk_path) {
                        Ok(Some(package_name)) => {
                            let mut packages = found_packages.lock().unwrap();
                            packages.push(package_name);
                            break;
                        },
                        Ok(None) => {
                            // 正常情况，跳出重试循环
                            break;
                        },
                        Err(_) => {
                            retries += 1;
                            if retries >= max_retries {
                                eprintln!("检查某个APK失败");
                            } else {
                                thread::sleep(Duration::from_millis(200 * retries as u64));
                            }
                        }
                    }
                }
                processed.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    let mut last_processed = 0;
    while last_processed < existing_apk_paths.len() as u64 {
        let current = processed.load(Ordering::Relaxed);
        pb.set_position(current);
        thread::sleep(Duration::from_millis(100));
        
        if current == last_processed {
            let mut all_done = true;
            for handle in &handles {
                if !handle.is_finished() {
                    all_done = false;
                    break;
                }
            }
            if all_done {
                break;
            }
        }
        
        last_processed = current;
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    pb.finish_with_message("扫描完成");
    
    let packages = Arc::try_unwrap(found_packages).unwrap().into_inner().unwrap();
    println!("找到 {} 个疑似隐藏应用列表的应用", packages.len());
    Ok(packages)
}

async fn hidemyapplist() -> AppResult {
    println!("开始查找隐藏应用列表...");

    let hma_package = if let Some(path) = find_hide_my_applist_dir() {
        if let Some(uid) = get_hma_uid(&path) {
            println!("找到 HMA UID: {}", uid);
            get_package_name_from_uid(&uid)
                .or_else(|| {
                    println!("无法通过 UID 获取包名，尝试使用 aapt 方法...");
                    find_hma_package_with_aapt().ok().and_then(|packages| select_package_from_list(&packages))
                })
        } else {
            println!("未找到 HMA UID，尝试使用 aapt 方法...");
            find_hma_package_with_aapt().ok().and_then(|packages| select_package_from_list(&packages))
        }
    } else {
        println!("未找到 HMA 目录，尝试使用 aapt 方法...");
        find_hma_package_with_aapt().ok().and_then(|packages| select_package_from_list(&packages))
    };

    if let Some(package_name) = hma_package {
        println!("找到的 HMA 包名: {}", package_name);
        configure_hma(&package_name).await
    } else {
        println!("无法找到HMA包名");
        download_config_to_sdcard().await
    }
}

fn select_package_from_list(packages: &[String]) -> Option<String> {
    if packages.is_empty() {
        None
    } else if packages.len() == 1 {
        Some(packages[0].clone())
    } else {
        println!("找到多个疑似隐藏应用列表的应用，请选择：");
        for (i, pkg) in packages.iter().enumerate() {
            println!("{}. {}", i + 1, pkg);
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("读取输入失败");

        match input.trim().parse::<usize>() {
            Ok(choice) if choice > 0 && choice <= packages.len() => {
                Some(packages[choice - 1].clone())
            },
            _ => {
                println!("无效选择，使用第一个应用");
                Some(packages[0].clone())
            }
        }
    }
}

async fn configure_hma(package_name: &str) -> AppResult {
    let file1 = "/data/cache/recovery/yshell/config.json";
    let file2 = format!("/data/data/{}/files/config.json", package_name);

    if let Err(e) = fs::create_dir_all("/data/cache/recovery/yshell/") {
        eprintln!("创建目录失败: {}", e);
        return download_config_to_sdcard().await;
    }

    if let Err(e) = download_file(
        "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/config.json".to_string(),
        true,
        Some(std::path::PathBuf::from(file1)),
        Some("4c8cf66c0f3d6359ab28562b04697440f78fc96db5043191fb9e28d083860a9c".to_string()),
    ).await {
        eprintln!("下载配置文件失败: {}", e);
        return download_config_to_sdcard().await;
    }

    if !Path::new(&file2).exists() {
        println!("未找到原配置文件，将配置文件下载到/sdcard/Download目录");
        return download_config_to_sdcard().await;
    }

    let new_config_content = match fs::read_to_string(file1) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("读取下载的配置文件失败: {}", e);
            return download_config_to_sdcard().await;
        }
    };

    if let Err(e) = fs::write(&file2, new_config_content) {
        eprintln!("写入原配置文件失败: {}", e);
        return download_config_to_sdcard().await;
    }

    println!("配置文件已成功更新");

    if let Err(e) = fs::remove_file(file1) {
        eprintln!("删除临时文件失败: {}", e);
    }
    
    Ok(())
}

async fn download_config_to_sdcard() -> AppResult {
    println!("下载下来的配置文件将存放在/sdcard/Download/文件夹里");
    println!("需要您手动到隐藏应用列表里点击还原配置");

    let file1 = "/sdcard/Download/隐藏应用列表配置.json";
    let config_hash = "b97c517369300d1c073cc4f49a0117912ee540f24161b2df306ed0e9f88fd426";
    
    download_file(
        "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/config.json".to_string(),
        true,
        Some(std::path::PathBuf::from(file1)),
        Some(config_hash.to_string()),
    ).await?;
    
    println!("已将配置文件保存在 /sdcard/Download/隐藏应用列表配置.json 中");
    Ok(())
}

fn recoverapplist() -> AppResult {
    let path_hma = match find_hide_my_applist_dir() {
        Some(path) => path,
        None => {
            eprintln!("未找到手机上的隐藏应用列表");
            return Err("未找到手机上的隐藏应用列表".into());
        }
    };

    let hma_uid = match get_hma_uid(&path_hma) {
        Some(uid) => uid,
        None => {
            eprintln!("无法获取HMA UID");
            return Err("无法获取HMA UID".into());
        }
    };

    let hma_package = match get_package_name_from_uid(&hma_uid) {
        Some(pkg) => pkg,
        None => {
            eprintln!("无法找到HMA包名");
            return Err("无法找到HMA包名".into());
        }
    };

    let backup_dir = "/sdcard/一键解决隐藏问题/";
    let file2 = format!("/data/data/{}/files/config.json", hma_package);
    let backup_file = format!("{}{}", backup_dir, std::path::Path::new(&file2).file_name().unwrap().to_string_lossy());

    if Path::new(&backup_file).exists() {
        if let Err(e) = fs::copy(&backup_file, &file2) {
            eprintln!("恢复备份失败: {}", e);
            return Err(e.into());
        }
        
        if let Err(e) = fs::remove_file(&backup_file) {
            eprintln!("删除备份文件失败: {}", e);
        }
        
        println!("恢复备份成功");
        Ok(())
    } else {
        eprintln!("错误：备份文件不存在！");
        Err("备份文件不存在".into())
    }
}

fn find_hide_my_applist_dir() -> Option<String> {
    if let Ok(entries) = fs::read_dir("/data/misc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with("hide_my_applist") {
                        return Some(path.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    None
}

fn get_hma_uid(path_hma: &str) -> Option<String> {
    let log_path = format!("{}/log/runtime.log", path_hma);

    if !Path::new(&log_path).exists() {
        eprintln!("日志文件不存在: {}", log_path);
        return None;
    }

    extract_uid_from_log(&log_path)
}

fn extract_uid_from_log(log_path: &str) -> Option<String> {
    let content = match fs::read_to_string(log_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("无法读取日志文件 {}: {}", log_path, e);
            return None;
        }
    };

    for line in content.lines() {
        if line.contains("Client uid") {
            println!("找到包含UID的行: {}", line);
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(uid) = parts.last() {
                if uid.chars().all(|c| c.is_ascii_digit()) {
                    println!("提取到 UID: {}", uid);
                    return Some(uid.to_string());
                } else {
                    eprintln!("找到的 UID 不是数字: {}", uid);
                }
            }
        }
    }
      
    eprintln!("在日志文件中未找到隐藏应用列表的UID");
    None
}

fn get_package_name_from_uid(uid: &str) -> Option<String> {
    println!("尝试通过 UID {} 获取包名", uid);
    
    match run_useful_tool_with_args("cmd", &["package", "list", "packages", "--uid", uid]) {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                println!("uid输出: {}", output_str);
                
                for line in output_str.lines() {
                    if line.starts_with("package:") {
                        let package_line = line.replace("package:", "");
                        let package_name = package_line.split_whitespace().next().unwrap_or("").to_string();
                        if !package_name.is_empty() {
                            println!("找到包名: {}", package_name);
                            return Some(package_name);
                        }
                    }
                }
                
                eprintln!("输出中没有找到有效的包名");
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                eprintln!("执行命令失败: {}", error_msg);
            }
            None
        },
        Err(e) => {
            eprintln!("执行命令失败: {}", e);
            None
        },
    }
}

fn run_useful_tool_with_args(tool_name: &str, args: &[&str]) -> AppResult<std::process::Output> {
    let tool_data = match tool_name {
        "cmd" => include_bytes!("binaries/cmd").as_slice(),
        "aapt" => include_bytes!("binaries/aapt").as_slice(),
        _ => return Err(format!("未知的工具名: {}", tool_name).into()),
    };

    let unique_id: u64 = rand::thread_rng().gen_range(0..1000000);
    let temp_file_name = format!("{}_{}", tool_name, unique_id);
    let temp_file = create_temp_tool(&temp_file_name, tool_data)?;
    
    let output = Command::new(&temp_file)
        .args(args)
        .output()?;

    fs::remove_file(&temp_file)?;
    
    Ok(output)
}

fn create_temp_tool(tool_name: &str, data: &[u8]) -> AppResult<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(tool_name);

    let mut file = File::create(&temp_file)?;
    file.write_all(data)?;
    file.flush()?;

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&temp_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_file, perms)?;
    }

    thread::sleep(Duration::from_millis(10));
    Ok(temp_file)
}

fn clean_modules_dirs() -> AppResult {
    let paths_to_clean = ["/data/adb/modules", "/data/adb/modules_update"];
    let exclude_dir = "AuroraNasa_Installer";

    for base_path in &paths_to_clean {
        if !Path::new(base_path).exists() {
            continue;
        }

        match fs::read_dir(base_path) {
            Ok(entries) => {
                let mut deleted_count = 0;
                let mut skipped_count = 0;

                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            eprintln!("读取目录项失败: {}", e);
                            continue;
                        }
                    };

                    let entry_path = entry.path();
                    
                    // 检查是否为目录
                    if entry_path.is_dir() {
                        let dir_name = match entry_path.file_name() {
                            Some(name) => name.to_string_lossy(),
                            None => continue,
                        };

                        // 跳过要保留的目录
                        if dir_name == exclude_dir {
                            skipped_count += 1;
                            continue;
                        }

                        // 删除目录
                        match fs::remove_dir_all(&entry_path) {
                            Ok(_) => {
                                deleted_count += 1;
                            },
                            Err(e) => {
                                eprintln!("删除目录 {} 失败: {}", entry_path.display(), e);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("读取目录 {} 失败: {}", base_path, e);
            }
        }
    }

    thread::sleep(Duration::from_millis(1400));
    Ok(())
}

fn deleter(delete_type: &str, path: &str, recursive: bool) -> AppResult {
    match delete_type {
        "file" => {
            if Path::new(path).exists() {
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("删除文件 {} 失败: {}", path, e);
                    return Err(e.into());
                }
                println!("已删除文件: {}", path);
            } else {
                println!("文件不存在，无需删除: {}", path);
            }
        },
        "dir" => {
            if Path::new(path).exists() {
                if let Err(e) = fs::remove_dir_all(path) {
                    eprintln!("删除目录 {} 失败: {}", path, e);
                    return Err(e.into());
                }
                println!("已删除目录: {}", path);
            } else {
                println!("目录不存在，无需删除: {}", path);
            }
        },
        "all" => {
            if Path::new(path).exists() && Path::new(path).is_dir() {
                let entries = match fs::read_dir(path) {
                    Ok(entries) => entries,
                    Err(e) => {
                        eprintln!("读取目录 {} 失败: {}", path, e);
                        return Err(e.into());
                    }
                };
                
                let mut deleted_count = 0;
                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(e) => {
                            eprintln!("读取目录项失败: {}", e);
                            continue;
                        }
                    };
                    let entry_path = entry.path();
                    
                    if entry_path.is_file() {
                        if let Err(e) = fs::remove_file(&entry_path) {
                            eprintln!("删除文件 {} 失败: {}", entry_path.display(), e);
                        } else {
                            deleted_count += 1;
                        }
                    } else if entry_path.is_dir() && recursive {
                        if let Err(e) = fs::remove_dir_all(&entry_path) {
                            eprintln!("删除目录 {} 失败: {}", entry_path.display(), e);
                        } else {
                            deleted_count += 1;
                        }
                    }
                }
                println!("已从 {} 中删除 {} 个项目", path, deleted_count);
            } else {
                println!("目录不存在或不是目录，无需删除: {}", path);
            }
        },
        _ => {
            return Err(format!("无效的删除类型: {}", delete_type).into());
        }
    }
    
    thread::sleep(Duration::from_millis(1400));
    Ok(())
}

fn run_useful_tool(tool_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let tool_args: Vec<&str> = if args.len() > 3 {
        args[3..].iter().map(|s| s.as_str()).collect()
    } else {
        Vec::new()
    };

    let tool_data = match tool_name {
        "cmd" => include_bytes!("binaries/cmd").as_slice(),
        _ => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("未知的工具名: {}", tool_name),
            )));
        }
    };

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(tool_name);

    {
        let mut file = File::create(&temp_file)?;
        file.write_all(tool_data)?;
        file.flush()?;
    }

    #[cfg(unix)]
    {        
        let mut perms = fs::metadata(&temp_file)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_file, perms)?;
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut command = Command::new(&temp_file);
    command.stdout(Stdio::inherit()).stderr(Stdio::inherit());

    for arg in tool_args {
        command.arg(arg);
    }
    
    let status = command.status()?;

    let _ = fs::remove_file(&temp_file);
    
    if status.success() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("工具执行失败，退出码: {:?}", status.code()),
        )))
    }
}

fn prop_module(module_id: &str, module_name: &str, system_prop_content: &str) {
    println!("正在生成模块");

    let temp_dir = format!("/data/cache/recovery/yshell/{}", module_id);
    if let Err(e) = fs::create_dir_all(&temp_dir) {
        eprintln!("创建临时目录失败: {}", e);
        std::process::exit(1);
    }

    let install_zip = "/data/cache/recovery/yshell/installmodule.zip";
    if Path::new(install_zip).exists() {
        if let Err(e) = fs::remove_file(install_zip) {
            eprintln!("删除旧安装包失败: {}", e);
        }
    }

    let module_prop_path = format!("{}/module.prop", temp_dir);
    let module_prop_content = format!(
        "id={}\nname={}\nversion=test\nversionCode=1.0\nauthor=酷安@yu13140\ndescription={}",
        module_id, module_name, module_name
    );
    
    if let Err(e) = fs::write(&module_prop_path, module_prop_content) {
        eprintln!("写入 module.prop 文件失败: {}", e);
        std::process::exit(1);
    }

    let system_prop_path = format!("{}/system.prop", temp_dir);
    if let Err(e) = fs::write(&system_prop_path, system_prop_content) {
        eprintln!("写入 system.prop 文件失败: {}", e);
        std::process::exit(1);
    }

    let customize_path = format!("{}/customize.sh", temp_dir);
    let customize_content = "SKIPUNZIP=0\nMODDIR=${0%/*}";
    if let Err(e) = fs::write(&customize_path, customize_content) {
        eprintln!("写入 customize.sh 文件失败: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = create_zip_from_dir(&temp_dir, install_zip) {
        eprintln!("创建 ZIP 文件失败: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        eprintln!("删除临时目录失败: {}", e);
    }

    println!("模块创建完成，已保存到: {}", install_zip);
    std::thread::sleep(std::time::Duration::from_millis(1400));
}

fn create_zip_from_dir(source_dir: &str, zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut zip, source_dir, "", options)?;
    
    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<File>,
    path: &str,
    base_path: &str,
    options: zip::write::FileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry_path.file_name().unwrap().to_string_lossy();
        
        let zip_path = if base_path.is_empty() {
            entry_name.to_string()
        } else {
            format!("{}/{}", base_path, entry_name)
        };
        
        if entry_path.is_file() {
            zip.start_file(zip_path, options)?;
            
            let mut file = File::open(&entry_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        } else if entry_path.is_dir() {
            zip.add_directory(&zip_path, options)?;
            add_dir_to_zip(zip, &entry_path.to_string_lossy(), &zip_path, options)?;
        }
    }
    
    Ok(())
}

fn get_system_prop(prop: &str) -> Option<String> {
    match Command::new("getprop").arg(prop).output() {
        Ok(output) => {
            if output.status.success() {
                let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if value.is_empty() {
                    None
                } else {
                    Some(value)
                }
            } else {
                None
            }
        },
        Err(_) => None,
    }
}

fn clean_lsplog() {
    let log_dirs = ["/data/adb/lspd/log", "/data/adb/lspd/log.old"];

    for dir in log_dirs.iter() {
        if Path::new(dir).exists() && Path::new(dir).is_dir() {
            match fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_file() {
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!("删除文件 {} 失败: {}", path.display(), e);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("读取目录 {} 失败: {}", dir, e);
                }
            }
        }
    }

    let environment = env::var("ENVIRONMENT").unwrap_or_default();

    if environment == "KernelSU" || environment == "SukiSU" {
        let resetprop_path = "/data/adb/ksu/bin/resetprop";
        if Path::new(resetprop_path).exists() {
            lsp_resetprop_commands(resetprop_path);
        } else {
            eprintln!("KSU resetprop 不存在: {}", resetprop_path);
        }
    } else {        
        lsp_resetprop_commands("resetprop");
    }
}

fn lsp_resetprop_commands(resetprop_path: &str) {
    let props = [
        "persist.logd.size",
        "persist.logd.size.crash",
        "persist.logd.size.main",
        "persist.logd.size.tag",
    ];
    
    for prop in props.iter() {
        match Command::new(resetprop_path)
            .arg("-n")
            .arg(prop)
            .arg("")
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!("重置属性 {} 失败", prop);
                }
            },
            Err(e) => {
                eprintln!("执行 resetprop 命令失败: {}", e);
                break;
            }
        }
    }
}

async fn boothash() -> Result<(), Box<dyn std::error::Error>> {
    let module_path = "/data/adb/modules/tricky_store";
    if !Path::new(module_path).exists() {
        println!("                                        ");
        println!("  正在运行脚本...失败");
        println!("  正在分析失败原因...未安装TrickyStore模块！");
        return Ok(());
    }

    let boot_hash = get_boot_hash().await?;
    
    println!("获取到的verifiedBootHash值: {}", boot_hash);

    let temp_dir = "/data/cache/recovery/yshell/Reset_BootHash";
    if let Err(e) = fs::create_dir_all(temp_dir) {
        eprintln!("创建临时目录失败: {}", e);
        return Err(e.into());
    }

    let module_prop_content = "id=Reset_BootHash\nname=重置哈希值\nversion=100\nversionCode=20240917\nauthor=yu13140\ndescription=辅助Tricky Store，实现增强BL隐藏。";
    
    if let Err(e) = fs::write(format!("{}/module.prop", temp_dir), module_prop_content) {
        eprintln!("写入 module.prop 文件失败: {}", e);
        return Err(e.into());
    }

    let service_content = format!("resetprop -n ro.boot.vbmeta.digest {}", boot_hash);
    
    if let Err(e) = fs::write(format!("{}/service.sh", temp_dir), service_content) {
        eprintln!("写入 service.sh 文件失败: {}", e);
        return Err(e.into());
    }

    #[cfg(unix)]
    {        
        if let Err(e) = fs::set_permissions(
            format!("{}/service.sh", temp_dir), 
            fs::Permissions::from_mode(0o755)
        ) {
            eprintln!("设置执行权限失败: {}", e);
        }
    }
    
    println!("                                        ");
    println!("把获取到的verifiedBootHash值添加到模块...完成");

    println!("正在压缩模块文件...");
    
    if let Err(e) = create_zip_from_dir(temp_dir, "/data/cache/recovery/yshell/installmodule.zip") {
        eprintln!("压缩模块失败: {}", e);
        return Err(e.into());
    }
    
    println!("模块已压缩保存到: /data/cache/recovery/yshell/installmodule.zip");
    
    println!("                                        ");
    println!("脚本执行完毕，请重启手机后查看牛头人应用！");
    
    Ok(())
}

async fn get_boot_hash() -> Result<String, Box<dyn std::error::Error>> {
    println!("正在下载service.apk...");
    let apk_path = "/data/cache/recovery/yshell/service.apk";

    match download_file(
        "https://github.com/yu13140/yuhideroot/raw/refs/heads/main/module/service.apk".to_string(),
        true,
        Some(std::path::PathBuf::from(apk_path)),
        None,
    ).await {
        Ok(_) => {
            println!("下载完成");
        },
        Err(e) => {
            eprintln!("下载失败: {}", e);
            return Err(e);
        }
    }

    println!("正在安装service.apk...");
    match run_useful_tool_with_args("cmd", &[
        "package", 
        "install", 
        "-r", 
        apk_path
    ]) {
        Ok(output) => {
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                eprintln!("安装失败: {}", error_msg);
                return Err(format!("APK安装失败: {}", error_msg).into());
            }
            println!("安装完成");
        },
        Err(e) => {
            eprintln!("安装失败: {}", e);
            return Err(e);
        }
    }

    println!("正在启动服务...");
    match run_useful_tool_with_args("cmd", &[
        "activity", 
        "start-foreground-service", 
        "-n", 
        "com.yu13140.verifiedboothash/.GetHashService"
    ]) {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("启动服务失败");
            }
        },
        Err(e) => {
            eprintln!("启动服务失败: {}", e);
        }
    }

    println!("等待服务完成工作...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let boot_hash_path = "/data/user/0/com.yu13140.verifiedboothash/files/verified_boot_hash.txt";
    let boot_hash = match fs::read_to_string(boot_hash_path) {
        Ok(content) => content.trim().to_string(),
        Err(e) => {
            eprintln!("读取boot_hash文件失败: {}", e);
            return Err(e.into());
        }
    };

    println!("正在卸载应用...");
    match run_useful_tool_with_args("cmd", &[
        "package", 
        "uninstall", 
        "com.yu13140.verifiedboothash"
    ]) {
        Ok(output) => {
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                eprintln!("卸载应用失败: {}", error_msg);
            } else {
                println!("应用卸载完成");
            }
        },
        Err(e) => {
            eprintln!("卸载应用失败: {}", e);
        }
    }
    
    Ok(boot_hash)
}

async fn nd_vbmeta() -> Result<(), Box<dyn std::error::Error>> {
    println!("正在生成模块");

    let temp_dir = "/data/cache/recovery/yshell/hide_vbmeta_error";
    if let Err(e) = fs::create_dir_all(temp_dir) {
        eprintln!("创建临时目录失败: {}", e);
        return Err(e.into());
    }

    let mut rng = rand::thread_rng();
    let random_value = rng.gen_range(1..=15);
    let vbmeta_size = 5504 + random_value * 1024;

    let boot_hash = get_boot_hash().await?;

    let mut service_content = String::new();
    service_content.push_str("#!/system/bin/sh\n\n");
    service_content.push_str("# 解决Native Detector提示检测到Boot状态异常问题\n");
    service_content.push_str("sleep 10\n\n");
    service_content.push_str("resetprop -n ro.boot.vbmeta.invalidate_on_error yes\n");
    service_content.push_str("resetprop -n ro.boot.vbmeta.hash_alg sha256\n");
    service_content.push_str(&format!("resetprop -n ro.boot.vbmeta.size {}\n", vbmeta_size));
    service_content.push_str("resetprop -n ro.boot.vbmeta.device_state locked\n");
    service_content.push_str("resetprop -n ro.boot.vbmeta.avb_version 1.2\n");
    
    if !boot_hash.is_empty() {
        service_content.push_str(&format!("resetprop -n ro.boot.vbmeta.digest {}\n", boot_hash));
    } else {
        eprintln!("无法获取boot哈希值");
        return Err("无法获取boot哈希值".into());
    }

    let service_path = format!("{}/service.sh", temp_dir);
    if let Err(e) = fs::write(&service_path, service_content) {
        eprintln!("写入 service.sh 文件失败: {}", e);
        return Err(e.into());
    }

    #[cfg(unix)]
    {
        if let Err(e) = fs::set_permissions(
            &service_path, 
            fs::Permissions::from_mode(0o755)
        ) {
            eprintln!("设置执行权限失败: {}", e);
        }
    
    let module_prop_path = format!("{}/module.prop", temp_dir);
    let module_prop_content = "id=hide_vbmeta_error\nname=解决Boot状态异常问题\nversion=test\nversionCode=2.0\nauthor=酷安@yu13140\ndescription=解决Native Detector提示检测到Boot状态异常问题";
    if let Err(e) = fs::write(&module_prop_path, module_prop_content) {
        eprintln!("写入 module.prop 文件失败: {}", e);
        return Err(e.into());
    }

    let customize_path = format!("{}/customize.sh", temp_dir);
    let customize_content = "SKIPUNZIP=0\nMODDIR=${0%/*}";
    if let Err(e) = fs::write(&customize_path, customize_content) {
        eprintln!("写入 customize.sh 文件失败: {}", e);
        return Err(e.into());
    }

    let install_zip = "/data/cache/recovery/yshell/installmodule.zip";
    if Path::new(install_zip).exists() {
        if let Err(e) = fs::remove_file(install_zip) {
            eprintln!("删除旧安装包失败: {}", e);
        }
    }

    if let Err(e) = create_zip_from_dir(temp_dir, install_zip) {
        eprintln!("创建 ZIP 文件失败: {}", e);
        return Err(e.into());
    }

    if let Err(e) = fs::remove_dir_all(temp_dir) {
        eprintln!("删除临时目录失败: {}", e);
    }

    println!("模块创建完成，已保存到: {}", install_zip);
    tokio::time::sleep(std::time::Duration::from_millis(1400)).await;
    Ok(())
    }
}
fn momo_tee() {
    let module_path = "/data/adb/modules/tricky_store";
    if !Path::new(module_path).exists() {
        eprintln!("你没有安装Tricky Store，是否安装此模块？");
    }
    
    let target_dir = "/data/adb/tricky_store";
    if !Path::new(target_dir).exists() {
        if let Err(e) = std::fs::create_dir_all(target_dir) {
            eprintln!("创建目录失败: {}", e);
            std::process::exit(1);
        }
    }
    
    match run_useful_tool_with_args("cmd", &["package", "list", "packages"]) {
        Ok(output) => {
            if output.status.success() {
                let packages_output = String::from_utf8_lossy(&output.stdout);
                
                let processed_packages: String = packages_output
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|line| line.replace("package:", ""))
                    .map(|line| if !line.is_empty() { format!("{}!\n", line) } else { line })
                    .collect();
                
                let target_file = format!("{}/target.txt", target_dir);
                if let Err(e) = std::fs::write(&target_file, processed_packages) {
                    eprintln!("写入文件失败: {}", e);
                    std::process::exit(1);
                }
                
                let tee_file = format!("{}/tee_status", target_dir);
                if let Err(e) = std::fs::write(&tee_file, "teeBroken=true") {
                    eprintln!("创建tee状态文件失败: {}", e);
                    std::process::exit(1);
                }
                
                println!("命令执行完成");
            } else {
                eprintln!("cmd package list packages 命令执行失败");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("执行cmd package list packages命令失败: {}", e);
            std::process::exit(1);
        }
    }
}

fn update_target_file() -> Result<(), Box<dyn std::error::Error>> {
    let target_file_path = "/data/adb/tricky_store/target.txt";

    if !Path::new(target_file_path).exists() {
        eprintln!("目标文件不存在: {}", target_file_path);
        return Err("目标文件不存在".into());
    }

    let content = fs::read_to_string(target_file_path)?;
    let mut lines: Vec<&str> = content.lines().collect();

    lines.retain(|&line| line.trim() != "luna.safe.luna");

    let has_hunter = lines.iter().any(|&line| line.trim() == "com.zhenxi.hunter");

    if !has_hunter {
        while let Some(last_line) = lines.last() {
            if last_line.trim().is_empty() {
                lines.pop();
            } else {
                break;
            }
        }
        lines.push("");
        lines.push("com.zhenxi.hunter");
    }

    let new_content = lines.join("\n");
    fs::write(target_file_path, new_content)?;

    println!("成功更新目标文件: {}", target_file_path);
    Ok(())
}

fn init_rc() -> Result<(), Box<dyn std::error::Error>> {
    println!("正在生成init.rc修复模块");

    let temp_dir = "/data/cache/recovery/yshell/Solve_initrc";
    if let Err(e) = fs::create_dir_all(temp_dir) {
        eprintln!("创建临时目录失败: {}", e);
        return Err(e.into());
    }

    let output = Command::new("getprop")
        .output()?;
    
    if !output.status.success() {
        return Err("执行getprop命令失败".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut service_commands = String::new();

    for line in output_str.lines() {
        if line.starts_with("[init.svc.") {
            let line = line.replace("[", "").replace("]", "");
            let parts: Vec<&str> = line.split(": ").collect();
            if parts.len() == 2 {
                let prop_name = parts[0];
                let prop_value = parts[1].trim();
                
                if prop_name == "init.svc.flash_recovery" {
                    service_commands.push_str(&format!("resetprop -n {}={}\n", prop_name, "stopped"));
                } else {
                    service_commands.push_str(&format!("resetprop -n {}={}\n", prop_name, prop_value));
                }
            }
        }
    }

    let service_path = format!("{}/service.sh", temp_dir);
    if let Err(e) = fs::write(&service_path, service_commands) {
        eprintln!("写入 service.sh 文件失败: {}", e);
        return Err(e.into());
    }

    let module_prop_path = format!("{}/module.prop", temp_dir);
    let module_prop_content = "id=Solve_initrc\nname=解决init.rc被修改问题\nversion=test\nversionCode=1.0\nauthor=酷安@yu13140\ndescription=解决init.rc被修改";
    if let Err(e) = fs::write(&module_prop_path, module_prop_content) {
        eprintln!("写入 module.prop 文件失败: {}", e);
        return Err(e.into());
    }

    let customize_path = format!("{}/customize.sh", temp_dir);
    let customize_content = "SKIPUNZIP=0\nMODDIR=${0%/*}";
    if let Err(e) = fs::write(&customize_path, customize_content) {
        eprintln!("写入 customize.sh 文件失败: {}", e);
        return Err(e.into());
    }

    let install_zip = "/data/cache/recovery/yshell/installmodule.zip";
    if let Err(e) = create_zip_from_dir(temp_dir, install_zip) {
        eprintln!("创建 ZIP 文件失败: {}", e);
        return Err(e.into());
    }

    if let Err(e) = fs::remove_dir_all(temp_dir) {
        eprintln!("删除临时目录失败: {}", e);
    }

    println!("模块创建完成，已保存到: {}", install_zip);
    std::thread::sleep(std::time::Duration::from_millis(1400));
    
    Ok(())
}

fn momo_addon() -> Result<(), Box<dyn std::error::Error>> {
    println!("高危选项！操作需要删除system分区里的addon.d文件夹");
    println!("删除这个文件夹，可能会使设备开机后不能写入system分区");
    println!("你确定要继续吗？(1.继续  2.退出): ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input == "2" {
        println!("你选择了退出");
        return Ok(());
    }

    println!("正在生成模块");

    let temp_dir = "/data/cache/recovery/yshell/delete_addond";
    if let Err(e) = fs::create_dir_all(temp_dir) {
        eprintln!("创建临时目录失败: {}", e);
        return Err(e.into());
    }

    let addon_dir = format!("{}/system/addon.d", temp_dir);
    if let Err(e) = fs::create_dir_all(&addon_dir) {
        eprintln!("创建 addon.d 目录失败: {}", e);
        return Err(e.into());
    }

    let replace_file = format!("{}/.replace", addon_dir);
    if let Err(e) = File::create(&replace_file) {
        eprintln!("创建 .replace 文件失败: {}", e);
        return Err(e.into());
    }

    let service_path = format!("{}/service.sh", temp_dir);
    let service_content = "sleep 10 && rm -rf /data/adb/modules/delete_addond/";
    if let Err(e) = fs::write(&service_path, service_content) {
        eprintln!("写入 service.sh 文件失败: {}", e);
        return Err(e.into());
    }

    let module_prop_path = format!("{}/module.prop", temp_dir);
    let module_prop_content = "id=sysaddons\nname=解决设备正在使用非原厂系统问题\nversion=test\nversionCode=1.0\nauthor=酷安@yu13140\ndescription=解决momo提示设备正在使用非原厂系统";
    if let Err(e) = fs::write(&module_prop_path, module_prop_content) {
        eprintln!("写入 module.prop 文件失败: {}", e);
        return Err(e.into());
    }

    let customize_path = format!("{}/customize.sh", temp_dir);
    let customize_content = "SKIPUNZIP=0\nMODDIR=${0%/*}";
    if let Err(e) = fs::write(&customize_path, customize_content) {
        eprintln!("写入 customize.sh 文件失败: {}", e);
        return Err(e.into());
    }

    let install_zip = "/data/cache/recovery/yshell/installmodule.zip";
    if Path::new(install_zip).exists() {
        if let Err(e) = fs::remove_file(install_zip) {
            eprintln!("删除旧安装包失败: {}", e);
        }
    }

    if let Err(e) = create_zip_from_dir(temp_dir, install_zip) {
        eprintln!("创建 ZIP 文件失败: {}", e);
        return Err(e.into());
    }

    if let Err(e) = fs::remove_dir_all(temp_dir) {
        eprintln!("删除临时目录失败: {}", e);
    }

    println!("模块创建完成，已保存到: {}", install_zip);
    Ok(())
}

fn momo_sdk() {
    println!("正在解决 非SDK接口的限制失效 问题");

    let settings_keys = [
        "hidden_api_policy",
        "hidden_api_policy_p_apps",
        "hidden_api_policy_pre_p_apps",
        "hidden_api_blacklist_exemptions",
        "hidden_api_blacklist_exe",
    ];

    for key in settings_keys.iter() {
        match Command::new("settings")
            .args(&["delete", "global", key])
            .output()
        {
            Ok(output) => {
                if !output.status.success() {
                    let error_output = String::from_utf8_lossy(&output.stderr);
                    if !error_output.contains("does not exist") {
                        eprintln!("删除设置 {} 失败: {}", key, error_output);
                    }
                }
            },
            Err(e) => {
                eprintln!("执行 settings 命令失败: {}", e);
            }
        }
    }
    
    println!("非SDK接口限制修复完成");
    std::thread::sleep(std::time::Duration::from_millis(1400));
}

fn holmes_sw() {
    println!("感谢酷安@but_you_forget提供的思路");
    println!("这可能需要一两分钟的时间，因机而异");

    let sw1_output = match Command::new("cmd")
        .args(&["package", "compile", "-m", "interpret-only", "-f", "com.android.settings"])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            eprintln!("执行 cmd package compile 失败: {}", e);
            std::process::exit(1);
        }
    };
 
    let sw1_output_str = String::from_utf8_lossy(&sw1_output.stdout);
    if sw1_output_str.to_lowercase().contains("failure") {
        eprintln!("❌ 执行出现错误！请私信作者报告错误");
        std::process::exit(1);
    }

    if let Err(e) = Command::new("cmd")
        .args(&["package", "compile", "-m", "interpret-only", "-f", "me.garfieldhan.holmes"])
        .output()
    {
        eprintln!("执行 cmd package compile 失败: {}", e);
        std::process::exit(1);
    }

    let cache_dirs = ["/data/dalvik-cache/arm", "/data/dalvik-cache/arm64"];
    for dir in cache_dirs.iter() {
        if Path::new(dir).exists() && Path::new(dir).is_dir() {
            match fs::read_dir(dir) {
                Ok(entries) => {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let path = entry.path();
                            if path.is_file() {
                                if let Err(e) = fs::remove_file(&path) {
                                    eprintln!("删除文件 {} 失败: {}", path.display(), e);
                                }
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("读取目录 {} 失败: {}", dir, e);
                }
            }
        }
    }

    if let Err(e) = Command::new("cmd")
        .args(&["package", "compile", "-m", "everything", "-f", "com.android.settings"])
        .output()
    {
        eprintln!("执行 cmd package compile 失败: {}", e);
        std::process::exit(1);
    }
 
    if let Err(e) = Command::new("cmd")
        .args(&["package", "compile", "-m", "everything", "-f", "me.garfieldhan.holmes"])
        .output()
    {
        eprintln!("执行 cmd package compile 失败: {}", e);
        std::process::exit(1);
    }
    
    println!("如果你想从根本解决问题，请换更高版本的LSPosed");
    std::thread::sleep(std::time::Duration::from_millis(1400));
}

fn holmes_9ff_check() -> Result<(), Box<dyn std::error::Error>> {
    let enableznctl = || -> Result<(), Box<dyn std::error::Error>> {
        let zygiskd_path = "/data/adb/modules/zygisksu/bin/zygiskd";

        Command::new(zygiskd_path)
            .arg("enforce-denylist")
            .arg("enabled")
            .output()?;

        if env::var("ENVIRONMENT").unwrap_or_default() == "Magisk" {
            Command::new("magisk")
                .args(&["denylist", "add", "me.garfieldhan.holmes"])
                .output()?;
        }

        Command::new(zygiskd_path)
            .arg("enforce-denylist")
            .arg("disabled")
            .output()?;
        
        Ok(())
    };

    if !Path::new("/data/adb/modules/zygisksu").exists() {
        println!("此方法依赖zygisk next模块，请去安装模块后再来执行");
        return Ok(());
    }

    let denylist_enforce_path = "/data/adb/zygisksu/denylist_enforce";
    if Path::new(denylist_enforce_path).exists() {
        if let Ok(content) = fs::read_to_string(denylist_enforce_path) {
            let lines: Vec<&str> = content.lines().collect();
            if !lines.is_empty() && lines[0] == "1" {
                Command::new("/data/adb/modules/zygisksu/bin/zygiskd")
                    .args(&["enforce-denylist", "disabled"])
                    .output()?;
            } else {
                enableznctl()?;
            }
        }
    } else {
        enableznctl()?;
    }

    Ok(())
}

fn holmes_9ff() -> Result<(), Box<dyn std::error::Error>> {
    let maphide_path = "/data/adb/modules/zygisk-maphide";
    if Path::new(maphide_path).exists() {
        println!("将要删除Zygisk Maphide模块(如果有的话)");
        if let Err(e) = fs::remove_dir_all(maphide_path) {
            eprintln!("删除 Zygisk Maphide 模块失败: {}", e);
        }
    }

    let zygisk_path = "/data/adb/modules/zygisksu";
    if Path::new(zygisk_path).exists() {
        let module_prop_path = format!("{}/module.prop", zygisk_path);
        if let Ok(content) = fs::read_to_string(&module_prop_path) {
            let mut version_code = 0;
            for line in content.lines() {
                if line.starts_with("versionCode=") {
                    if let Ok(v) = line.split('=').nth(1).unwrap_or("0").parse::<i32>() {
                        version_code = v;
                    }
                    break;
                }
            }
            
            if version_code >= 512 {
                holmes_9ff_check()?;
            } else {
                println!("你的Zygisk Next模块不是最新");
                println!("安装模块后重启，打开Holmes应用，如果仍有9ff请再执行一次脚本");
            }
        }
    } else {
        println!("安装模块后重启，打开Holmes应用，如果仍有9ff请再执行一次脚本");
    }

    Ok(())
}

fn clean_package_dex(package_name: &str) {
    match run_useful_tool_with_args("cmd", &["package", "list", "packages", "-f"]) {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let mut found = false;
                
                for line in output_str.lines() {
                    if line.contains(package_name) {
                        found = true;
                        let cleaned_line = line.replace("package:", "");
                        
                        let parts: Vec<&str> = cleaned_line.split("base").collect();
                        if !parts.is_empty() {
                            let path = parts[0].trim();
                            
                            if Path::new(path).exists() {
                                match Command::new("find")
                                    .arg(path)
                                    .args(&["-type", "f", "-name", "*.*dex"])
                                    .args(&["-exec", "rm", "{}", ";"])
                                    .output()
                                {
                                    Ok(output) => {
                                        if output.status.success() {
                                            println!("已清理 {} 目录中的 dex 文件", path);
                                        } else {
                                            let error_output = String::from_utf8_lossy(&output.stderr);
                                            eprintln!("清理 dex 文件失败: {}", error_output);
                                        }
                                    },
                                    Err(e) => {
                                        eprintln!("执行 find 命令失败: {}", e);
                                    }
                                }
                            } else {
                                eprintln!("路径不存在: {}", path);
                            }
                        }
                        break;
                    }
                }
                
                if !found {
                    eprintln!("未找到包名为 {} 的应用", package_name);
                }
            } else {
                eprintln!("cmd package list packages -f 命令执行失败");
            }
        },
        Err(e) => {
            eprintln!("执行 cmd package list packages -f 命令失败: {}", e);
        }
    }
}

fn cts_fix() {
    let pif_path = "/data/adb/modules/playintegrityfix/pif.json";
    if !Path::new(pif_path).exists() {
        eprintln!("您未刷入playintegrityfix模块，请使用更新模块功能刷入此模块");
        std::process::exit(1);
    }

    let fingerprint = get_system_prop("ro.system.build.fingerprint").unwrap_or_default();

    let content = match fs::read_to_string(pif_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("读取pif.json失败: {}", e);
            std::process::exit(1);
        }
    };

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if lines.len() >= 2 {
        let new_fingerprint = format!("  \"FINGERPRINT\": \"{}\",", fingerprint);
        lines[1] = new_fingerprint;
    }

    if let Err(e) = fs::write(pif_path, lines.join("\n")) {
        eprintln!("写入pif.json失败: {}", e);
        std::process::exit(1);
    }
    
    println!("CTS修复完成");
    std::thread::sleep(std::time::Duration::from_millis(1400));
}

fn hunter_miui() {
    println!("目前仅适用于米系手机(小米，红米)");
    
    match run_useful_tool_with_args("cmd", &["package", "disable", "--user", "0", "com.miui.securitycenter/com.xiaomi.security.xsof.MiSafetyDetectService"]) {
        Ok(output) => {
            if output.status.success() {
                println!("已禁用MIUI安全检测服务");
            } else {
                eprintln!("禁用服务失败");
            }
        },
        Err(e) => {
            eprintln!("执行cmd package disable命令失败: {}", e);
        }
    }
    
    std::thread::sleep(std::time::Duration::from_millis(1400));
}

fn shamiko_modules() {
    let shamiko_path = "/data/adb/shamiko";
    
    if !Path::new(shamiko_path).exists() {
        eprintln!("你没有安装Shamiko!");
        eprintln!("请到更新模块功能里安装Shamiko");
        std::process::exit(0);
    }
    
    let whitelist_path = format!("{}/whitelist", shamiko_path);
    let whitelist_exists = Path::new(&whitelist_path).exists();
    
    if whitelist_exists {
        if let Err(e) = fs::remove_file(&whitelist_path) {
            eprintln!("删除白名单文件失败: {}", e);
        } else {
            println!("Shamiko已设置黑名单模式");
        }
    } else {
        if let Err(e) = fs::File::create(&whitelist_path) {
            eprintln!("创建白名单文件失败: {}", e);
        } else {
            println!("Shamiko已设置白名单模式");
        }
    }
}

fn nd_magicmount() {
    let zygisk_path = "/data/adb/modules/zygisksu";
    
    if !Path::new(zygisk_path).exists() {
        eprintln!("请去更新模块功能里，下载最新的Zygisk Next");
        std::process::exit(1);
    }
    
    let module_prop_path = format!("{}/module.prop", zygisk_path);
    let content = match fs::read_to_string(&module_prop_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("读取module.prop失败: {}", e);
            std::process::exit(1);
        }
    };

    let mut version = 0;
    for line in content.lines() {
        if line.starts_with("versionCode=") {
            if let Ok(v) = line.split('=').nth(1).unwrap_or("0").parse::<i32>() {
                version = v;
            }
            break;
        }
    }
    
    if version >= 512 {
        let no_mount_path = "/data/adb/zygisksu/no_mount_znctl";
        if let Err(e) = fs::File::create(no_mount_path) {
            eprintln!("创建no_mount_znctl文件失败: {}", e);
        } else {
            println!("已创建no_mount_znctl文件");
        }
    } else {
        eprintln!("请去更新模块功能里，下载最新的Zygisk Next");
        std::process::exit(1);
    }
    
    std::thread::sleep(std::time::Duration::from_millis(1400));
}

fn momo(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match profile_name {
        "tee" => {
            momo_tee();
            Ok(())
        },
        "systemmount" => {
            prop_module("Solve_systemmout", "解决数据未加密，挂载参数被修改的问题", "ro.crypto.state=encrypted");
            Ok(())
        },
        "development" => {
            prop_module("Solve_Development", "解决处于调试环境的问题", "ro.crypto.state=encrypted");
            Ok(())
        },
        "sdk" => {
            momo_sdk();
            Ok(())
        },
        "addon" => {
            momo_addon()?;
            Ok(())
        },
        _ => {
            eprintln!("未知的参数: {}", profile_name);
            print_help();
            Err("未知参数".into())
        }
    }
}

async fn nativetest(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match profile_name {
        "futile10" => {
            clean_package_dex("icu.nullptr.nativetest");
            Ok(())
        }
        "boothash" => {
            boothash().await?;
            Ok(())
        }
        _ => {
            eprintln!("未知的参数: {}", profile_name);
            print_help();
            Err("未知参数".into())
        }
    }
}

fn holmes(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match profile_name {
        "somethingwrong" => {
            holmes_sw();
            Ok(())
        },
        "9ff" => holmes_9ff(),
        "development" => {
            prop_module("Solve_Development", "解决处于调试环境的问题", "ro.crypto.state=encrypted");
            Ok(())
        },
        _ => {
            eprintln!("未知的参数: {}", profile_name);
            print_help();
            Err("未知参数".into())
        }
    }
}

fn hunter(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match profile_name {
        "shizuku" => {
            let _ = deleter("file", "/data/local/tmp/shizuku_starter", false);
            let _ = deleter("dir", "/data/local/tmp/shizuku", false);
            Ok(())
        }
        "manager" => {
            hunter_miui();
            Ok(())
        }
        _ => {
            eprintln!("未知的参数: {}", profile_name);
            print_help();
            Err("未知参数".into())
        }
    }
}

async fn nativedetector(profile_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    match profile_name {
        "vbmeta" => {
            nd_vbmeta().await?;
            Ok(())
        }
        "magicmount" => {
            nd_magicmount();
            Ok(())
        }
        "lsp5" => {
            clean_package_dex("com.reveny.nativecheck");
            Ok(())
        }
        _ => {
            eprintln!("未知的参数: {}", profile_name);
            print_help();
            Err("未知参数".into())
        }
    }
}