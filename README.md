# NetProbe 🔍

**A surgical network diagnostic tool for DevOps & SysAdmins.**
*Written in Rust. Blazingly fast. Universal.*

NetProbe replaces the "Ping -\> Curl -\> Browser" dance with a single command that analyzes the entire connection chain: **DNS Resolution**, **TCP Handshake**, and **HTTP/HTTPS Status**.



## 🚀 Features

- **🧅 Layered Analysis**: Instantly isolate faults. Is it a DNS typo? A Firewall blocking the port (TCP)? or a Server Error (HTTP)?
- **🤖 JSON Output**: Full support for automation, CI/CD pipelines, and monitoring scripts (`--json`).
- **🛡️ Secure & Portable**: Built with `rustls` (no OpenSSL dependency hell). Single static binary.
- **⚡️ Universal**: Works on IPs, Domains, Localhost, and custom ports (e.g., `localhost:8080`).

-----

## 📦 Installation

To use `netprobe` globally in your terminal (like `ls` or `git`), follow the instructions for your OS.

### Option 1: Download Binary (Recommended)

Go to the [**Releases Page**](https://github.com/ton-pseudo/netprobe/releases) and download the file for your system.

#### 🍎 macOS / 🐧 Linux

1.  Download the binary.
2.  Make it executable and move it to your path:

<!-- end list -->

```bash
# Example assuming you are in the download folder
chmod +x netprobe
sudo mv netprobe /usr/local/bin/
```

3.  Run it: `netprobe google.com`

#### 🪟 Windows

1.  Download `netprobe.exe`.
2.  Create a folder (e.g., `C:\Tools`) and move the `.exe` there.
3.  Add `C:\Tools` to your **Environment Variables (PATH)**.
4.  Open a new PowerShell/CMD and run: `netprobe google.com`

### Option 2: Build from Source (Rust Developers)

```bash
cargo install --path .
```

-----

## 🛠 Usage & Examples

Here are the concrete ways to use NetProbe in your daily workflow.

### 1\. Basic Diagnostic (The "Why is it down?" check)

Ideal for quick checks on websites or servers.

```bash
netprobe github.com
```

*Output:* Checks DNS, connects to port 443, and verifies SSL/HTTP status.

### 2\. Follow Redirects (`-f`)

By default, NetProbe reports redirects (301/302) as warnings. Use `-f` to follow the chain to the final destination.

```bash
# google.com redirects to www.google.com
netprobe google.com -f
```

### 3\. Debugging Docker / Local Services

Test specific ports on your machine or local network.

```bash
# Check if your local Postgres container is reachable
netprobe localhost:5432
```

*Note: HTTP check will fail on database ports, but if TCP is ✅, your container is running fine.*

### 4\. Fast Timeout (`-t`)

Don't wait 30 seconds for a hanging server. Force a quick fail (useful for firewalls).

```bash
# Fail if connection takes more than 1 second
netprobe 192.168.1.55 -t 1
```

### 5\. DevOps & Scripting Mode (`--json`) 🤖

Output raw JSON data to pipe into other tools (like `jq` or Python scripts).

```bash
netprobe api.stripe.com --json
```

**Example: Alert if latency is too high**

```bash
# Get HTTP latency in ms
netprobe google.com --json | jq .http.latency_ms
```

-----

## 📚 Command Line Reference

| Argument | Short | Description | Default |
| :--- | :---: | :--- | :---: |
| `target` | - | The URL, IP, or Domain to test | Required |
| `--json` | `-j` | Output results in JSON format | `false` |
| `--timeout` | `-t` | Connection timeout in seconds | `5` |
| `--follow-redirects` | `-f` | Follow HTTP 3xx redirects | `false` |

-----

## 📸 Output Comparison

### Standard Human Output

```text
🔍 Probing Target: https://github.com
--------------------------------------------------
1. DNS Resolution   ✅ 140.82.121.3 (20.35ms)
2. TCP Handshake    ✅ Port 443 Open (27.18ms)
3. HTTP Request     ✅ Status: 200 OK (150.22ms)
--------------------------------------------------
```

### JSON Output (`--json`)

```json
{
  "target": "https://github.com",
  "timestamp": "2025-12-11T16:00:18+01:00",
  "dns": {
    "status": "ok",
    "ip": "140.82.121.3",
    "latency_ms": 20.35
  },
  "tcp": {
    "status": "ok",
    "port": 443,
    "latency_ms": 27.18
  },
  "http": {
    "status_code": 200,
    "latency_ms": 150.22,
    "headers": {
      "server": "github.com"
    }
  }
}
```

## 📝 License

Distributed under the MIT License. See `LICENSE` for more information.
