use clap::Parser;
use colored::*;
use serde::Serialize;
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::{Duration, Instant};
use url::Url;

// --- JSON Data Structures ---
// These structures ensure the JSON output is standardized and predictable.

#[derive(Serialize)]
struct ProbeResult {
    target: String,
    timestamp: String,
    dns: DnsResult,
    tcp: TcpResult,
    http: HttpResult,
}

#[derive(Serialize)]
struct DnsResult {
    status: String, // "ok" | "error"
    ip: Option<String>,
    latency_ms: Option<f64>,
    error: Option<String>,
}

#[derive(Serialize)]
struct TcpResult {
    status: String,
    port: u16,
    latency_ms: Option<f64>,
    error: Option<String>,
}

#[derive(Serialize)]
struct HttpResult {
    status_code: Option<u16>,
    latency_ms: Option<f64>,
    headers: Option<HashMap<String, String>>,
    error: Option<String>,
}

// --- CLI Arguments ---
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The target URL or IP (e.g., google.com, 192.168.1.1)
    target: String,

    /// Output results in raw JSON format (ideal for scripting/pipelines)
    #[arg(long, short = 'j')]
    json: bool,

    /// Set a custom timeout in seconds
    #[arg(long, short = 't', default_value_t = 5)]
    timeout: u64,

    /// Follow HTTP redirects (3xx)
    #[arg(long, short = 'f', default_value_t = false)]
    follow_redirects: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // 1. Input Sanitization & Parsing
    // Automatically prepend https:// if no scheme is provided for convenience.
    let target_input = if !args.target.contains("://") {
        format!("https://{}", args.target)
    } else {
        args.target.clone()
    };

    let url = match Url::parse(&target_input) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("{} Invalid URL format: {}", "✖".red(), e);
            std::process::exit(1);
        }
    };

    let host = url.host_str().unwrap_or("").to_string();
    // Default ports: 443 for https, 80 for http, or use specified port
    let port = url.port_or_known_default().unwrap_or(443);

    // Initialize result structure
    let mut probe_data = ProbeResult {
        target: target_input.clone(),
        timestamp: chrono::Local::now().to_rfc3339(),
        dns: DnsResult { status: "pending".to_string(), ip: None, latency_ms: None, error: None },
        tcp: TcpResult { status: "pending".to_string(), port, latency_ms: None, error: None },
        http: HttpResult { status_code: None, latency_ms: None, headers: None, error: None },
    };

    // UI Header (only if not in JSON mode)
    if !args.json {
        println!("\n🔍 Probing Target: {}", target_input.bold().cyan());
        println!("{}", "--------------------------------------------------".dimmed());
    }

    // --- STEP 1: DNS Resolution ---
    let start_dns = Instant::now();
    let socket_addr_str = format!("{}:{}", host, port);
    // Blocking call is acceptable here for simplicity in a CLI tool
    let ip_lookup = socket_addr_str.to_socket_addrs();
    let dns_duration = start_dns.elapsed().as_secs_f64() * 1000.0;

    let resolved_ip = match ip_lookup {
        Ok(mut addrs) => {
            if let Some(ip) = addrs.next() {
                probe_data.dns.status = "ok".to_string();
                probe_data.dns.ip = Some(ip.ip().to_string());
                probe_data.dns.latency_ms = Some(dns_duration);

                if !args.json {
                    println!("1. DNS Resolution   {} {} ({:.2}ms)", "✅".green(), ip.ip().to_string().yellow(), dns_duration);
                }
                Some(ip)
            } else {
                probe_data.dns.status = "error".to_string();
                probe_data.dns.error = Some("No IP found".to_string());
                if !args.json { println!("1. DNS Resolution   {} Failed: No IP found", "❌".red()); }
                None
            }
        },
        Err(e) => {
            probe_data.dns.status = "error".to_string();
            probe_data.dns.error = Some(e.to_string());
            if !args.json { println!("1. DNS Resolution   {} Error: {}", "❌".red(), e); }
            None
        }
    };

    // --- STEP 2: TCP Handshake ---
    if let Some(ip) = resolved_ip {
        let start_tcp = Instant::now();
        // Attempt TCP connection with timeout
        match std::net::TcpStream::connect_timeout(&ip, Duration::from_secs(args.timeout)) {
            Ok(_) => {
                let tcp_duration = start_tcp.elapsed().as_secs_f64() * 1000.0;
                probe_data.tcp.status = "ok".to_string();
                probe_data.tcp.latency_ms = Some(tcp_duration);

                if !args.json {
                    println!("2. TCP Handshake    {} Port {} Open ({:.2}ms)", "✅".green(), port, tcp_duration);
                }
            },
            Err(e) => {
                probe_data.tcp.status = "error".to_string();
                probe_data.tcp.error = Some(e.to_string());

                if !args.json {
                    println!("2. TCP Handshake    {} Connection Refused or Timeout", "❌".red());
                }
                // We continue to HTTP check even if TCP fails, just in case of weird proxy setups,
                // though usually it will fail there too.
            }
        }
    }

    // --- STEP 3: HTTP/HTTPS Request ---
    let start_http = Instant::now();

    // Configure Redirect Policy
    let redirect_policy = if args.follow_redirects {
        reqwest::redirect::Policy::limited(10)
    } else {
        reqwest::redirect::Policy::none()
    };

    // Build Client with Timeout and Policy
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .redirect(redirect_policy)
        .user_agent("NetProbe/1.0") // Good practice to identify your tool
        .build()
        .unwrap_or_default();

    // Send HEAD request (lighter than GET)
    match client.head(&target_input).send().await {
        Ok(response) => {
            let http_duration = start_http.elapsed().as_secs_f64() * 1000.0;
            let status = response.status();

            probe_data.http.status_code = Some(status.as_u16());
            probe_data.http.latency_ms = Some(http_duration);

            // Capture relevant headers
            let mut headers_map = HashMap::new();
            if let Some(h) = response.headers().get("server") {
                headers_map.insert("server".to_string(), h.to_str().unwrap_or("unknown").to_string());
            }
            if let Some(h) = response.headers().get("content-type") {
                headers_map.insert("content-type".to_string(), h.to_str().unwrap_or("unknown").to_string());
            }
            probe_data.http.headers = Some(headers_map);

            if !args.json {
                if status.is_success() {
                    println!("3. HTTP Request     {} Status: {} ({:.2}ms)", "✅".green(), status, http_duration);
                } else if status.is_redirection() {
                    println!("3. HTTP Request     {} Status: {} (Redirect) ({:.2}ms)", "⚠️".yellow(), status, http_duration);
                } else {
                    println!("3. HTTP Request     {} Status: {} ({:.2}ms)", "❌".red(), status, http_duration);
                }
            }
        },
        Err(e) => {
            probe_data.http.error = Some(e.to_string());
            if !args.json {
                println!("3. HTTP Request     {} Error: {}", "❌".red(), e);
            }
        }
    }

    // Final Output
    if args.json {
        // Print raw JSON for piping
        let json_output = serde_json::to_string_pretty(&probe_data).unwrap();
        println!("{}", json_output);
    } else {
        println!("{}", "--------------------------------------------------".dimmed());
    }
}