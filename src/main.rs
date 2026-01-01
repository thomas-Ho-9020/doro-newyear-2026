use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::thread;
use std::time::Duration;
use time::{OffsetDateTime, format_description};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("🚀 粉紅 Doro 2026 跨年伺服器啟動！（香港實時版）");
    println!("   本機訪問：http://127.0.0.1:7878");
    println!("   局域網訪問：http://你嘅Pi_IP:7878");
    println!("   例如：http://192.168.1.123:7878\n");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_connection(stream);
                });
            }
            Err(e) => {
                eprintln!("[{}] ❌ 連線錯誤: {}", current_time(), e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let peer_addr = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "未知IP".to_string(),
    };

    let mut buffer = [0; 1024];
    let read_result = stream.read(&mut buffer);

    // 日誌：新連線開始
    println!("[{}] 🆕 新連線來自 {}", current_time(), peer_addr);

    if read_result.is_err() {
        println!("[{}] 🔌 {} 連線異常中斷", current_time(), peer_addr);
        return;
    }

    let request = String::from_utf8_lossy(&buffer[..]);
    let first_line = request.lines().next().unwrap_or("");

    // 解析路徑
    let path = if first_line.starts_with("GET / ") || first_line == "GET / HTTP/1.1" {
        "/index.html".to_string()
    } else if let Some(req_path) = first_line.strip_prefix("GET ") {
        let end = req_path.find(' ').unwrap_or(req_path.len());
        let p = req_path[..end].to_string();
        if p.is_empty() || p == "/" {
            "/index.html".to_string()
        } else {
            p
        }
    } else {
        "/index.html".to_string()
    };

    // 長路徑截斷顯示
    let display_path = if path.len() > 40 {
        format!("{}...", &path[..37])
    } else {
        path.clone()
    };

    println!("[{}] {} 請求 {}", current_time(), peer_addr, display_path);

    // 回應處理
    let mut success = false;
    if path == "/" || path == "/index.html" {
        if let Ok(html) = fs::read_to_string("index.html") {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}",
                html
            );
            if stream.write_all(response.as_bytes()).is_ok() {
                success = true;
            }
        }
    } else {
        let file_path = format!("static{}", path);
        if Path::new(&file_path).exists() {
            if let Ok(contents) = fs::read(&file_path) {
                let mime = get_mime_type(&path);
                let header = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\n\r\n", mime);
                if stream.write_all(header.as_bytes()).is_ok() && stream.write_all(&contents).is_ok() {
                    success = true;
                }
            }
        }
    }

    if success {
        println!("[{}] {} ← 成功傳送 {}", current_time(), peer_addr, display_path);
    } else if !path.ends_with("/favicon.ico") {
        println!("[{}] {} ← 404 或傳送失敗 {}", current_time(), peer_addr, display_path);
    }

    let _ = stream.flush();
    thread::sleep(Duration::from_millis(50));

    // 日誌：連線結束
    println!("[{}] 🔌 {} 連線結束（資源傳送完成）", current_time(), peer_addr);
}

// 實時香港時間（自動處理跨年）
fn current_time() -> String {
    let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .unwrap();

    let now = OffsetDateTime::now_local()
        .unwrap_or_else(|_| OffsetDateTime::now_utc() + time::Duration::hours(8));

    now.format(&format).unwrap()
}

// MIME 類型
fn get_mime_type(path: &str) -> &'static str {
    if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    }else if path.ends_with(".txt") {
        "txt"
    }else {
        "application/octet-stream"
    }
}