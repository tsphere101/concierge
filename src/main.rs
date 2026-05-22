use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};

// ─── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = msg["method"].as_str().unwrap_or("");
        let id = msg.get("id");

        match method {
            "initialize" => {
                send_result(
                    &stdout,
                    id,
                    json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": {} },
                        "serverInfo": { "name": "concierge", "version": "0.1.0" }
                    }),
                );
            }
            "notifications/initialized" => {}
            "tools/list" => {
                let tools: Vec<Value> = tools()
                    .into_iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.schema
                        })
                    })
                    .collect();
                send_result(&stdout, id, json!({ "tools": tools }));
            }
            "tools/call" => {
                let name = msg["params"]["name"].as_str().unwrap_or("");
                let args = &msg["params"]["arguments"];
                let result = handle_tool(name, args);
                let mut response = json!({
                    "content": [{ "type": "text", "text": result.text }]
                });
                if result.is_error {
                    response["isError"] = json!(true);
                }
                send_result(&stdout, id, response);
            }
            _ => {
                if let Some(id) = id {
                    send_error(
                        &stdout,
                        id,
                        -32601,
                        format!("Method not found: {}", method),
                    );
                }
            }
        }
    }
}

// ─── Tool registry ───────────────────────────────────────────────────────────

struct ToolDef {
    name: &'static str,
    description: &'static str,
    schema: Value,
    handler: fn(&Value) -> ToolResult,
}

struct ToolResult {
    text: String,
    is_error: bool,
}

impl ToolResult {
    fn ok(text: impl Into<String>) -> Self {
        ToolResult {
            text: text.into(),
            is_error: false,
        }
    }
    fn err(text: impl Into<String>) -> Self {
        ToolResult {
            text: text.into(),
            is_error: true,
        }
    }
}

fn tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "wake",
            description: "Keep the Mac awake for N hours using Amphetamine",
            schema: json!({
                "type": "object",
                "properties": {
                    "hours": { "type": "number", "description": "Number of hours (default 1)" }
                }
            }),
            handler: wake,
        },
        ToolDef {
            name: "open",
            description: "Open a file path or URL with the default application",
            schema: json!({
                "type": "object",
                "properties": {
                    "target": { "type": "string", "description": "File path or URL to open" }
                },
                "required": ["target"]
            }),
            handler: open_tool,
        },
        ToolDef {
            name: "reveal",
            description: "Reveal a file or folder in Finder",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File or folder path to reveal in Finder" }
                },
                "required": ["path"]
            }),
            handler: reveal,
        },
        ToolDef {
            name: "volume",
            description: "Control system output volume",
            schema: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["set", "get", "mute", "unmute"],
                        "description": "Action to perform"
                    },
                    "level": {
                        "type": "number",
                        "description": "Volume level 0-100 (for set action)"
                    }
                },
                "required": ["action"]
            }),
            handler: volume,
        },
        ToolDef {
            name: "battery",
            description: "Get battery status",
            schema: json!({
                "type": "object",
                "properties": {}
            }),
            handler: battery,
        },
        ToolDef {
            name: "say",
            description: "Speak text aloud using text-to-speech",
            schema: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to speak" },
                    "voice": { "type": "string", "description": "Voice name (optional)" },
                    "rate": { "type": "number", "description": "Words per minute (optional)" }
                },
                "required": ["text"]
            }),
            handler: say_tool,
        },
        ToolDef {
            name: "clipboard-read",
            description: "Read text from the clipboard",
            schema: json!({
                "type": "object",
                "properties": {}
            }),
            handler: clipboard_read,
        },
        ToolDef {
            name: "clipboard-write",
            description: "Write text to the clipboard",
            schema: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text to copy to clipboard" }
                },
                "required": ["text"]
            }),
            handler: clipboard_write,
        },
    ]
}

fn handle_tool(name: &str, args: &Value) -> ToolResult {
    for tool in tools() {
        if tool.name == name {
            return (tool.handler)(args);
        }
    }
    ToolResult::err(format!("Unknown tool: {}", name))
}

// ─── Tool handlers ───────────────────────────────────────────────────────────

fn wake(args: &Value) -> ToolResult {
    let hours = args["hours"].as_f64().unwrap_or(1.0) as u64;
    let script = format!(
        r#"tell application "Amphetamine" to start new session with options {{duration:{}, interval:hours, displaySleepAllowed:false}}"#,
        hours
    );
    match osascript(&script) {
        Ok(_) => ToolResult::ok(format!("Amphetamine session started for {} hour(s)", hours)),
        Err(e) => ToolResult::err(e),
    }
}

fn open_tool(args: &Value) -> ToolResult {
    let target = args["target"].as_str().unwrap_or("");
    if target.is_empty() {
        return ToolResult::err("No target provided");
    }
    match Command::new("open").arg(target).output() {
        Ok(out) if out.status.success() => ToolResult::ok(format!("Opened: {}", target)),
        Ok(out) => ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => ToolResult::err(format!("Error: {}", e)),
    }
}

fn reveal(args: &Value) -> ToolResult {
    let path = args["path"].as_str().unwrap_or("");
    if path.is_empty() {
        return ToolResult::err("No path provided");
    }
    if !Path::new(path).exists() {
        return ToolResult::err(format!("Path not found: {}", path));
    }
    match Command::new("open").arg("-R").arg(path).output() {
        Ok(out) if out.status.success() => ToolResult::ok(format!("Revealed: {}", path)),
        Ok(out) => ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => ToolResult::err(format!("Error: {}", e)),
    }
}

fn volume(args: &Value) -> ToolResult {
    let action = args["action"].as_str().unwrap_or("get");
    match action {
        "set" => {
            let level = args["level"].as_f64().unwrap_or(50.0) as i64;
            let level = level.clamp(0, 100);
            match osascript(&format!("set volume output volume {}", level)) {
                Ok(_) => ToolResult::ok(format!("Volume set to {}%", level)),
                Err(e) => ToolResult::err(e),
            }
        }
        "get" => match osascript("output volume of (get volume settings)") {
            Ok(vol) => ToolResult::ok(format!("Volume: {}%", vol.trim())),
            Err(e) => ToolResult::err(e),
        },
        "mute" => match osascript("set volume with output muted") {
            Ok(_) => ToolResult::ok("Muted"),
            Err(e) => ToolResult::err(e),
        },
        "unmute" => match osascript("set volume without output muted") {
            Ok(_) => ToolResult::ok("Unmuted"),
            Err(e) => ToolResult::err(e),
        },
        _ => ToolResult::err(format!(
            "Unknown action: {}. Use set/get/mute/unmute",
            action
        )),
    }
}

fn battery(_args: &Value) -> ToolResult {
    match Command::new("pmset").arg("-g").arg("batt").output() {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            ToolResult::ok(text.trim().to_string())
        }
        Err(e) => ToolResult::err(format!("Failed to get battery status: {}", e)),
    }
}

fn say_tool(args: &Value) -> ToolResult {
    let text = args["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return ToolResult::err("No text provided");
    }

    let mut cmd = Command::new("say");
    if let Some(voice) = args["voice"].as_str() {
        cmd.arg("-v").arg(voice);
    }
    if let Some(rate) = args["rate"].as_f64() {
        cmd.arg("-r").arg(rate.to_string());
    }
    cmd.arg(text);

    match cmd.spawn() {
        Ok(_) => ToolResult::ok(format!("Speaking: {}", text)),
        Err(e) => ToolResult::err(format!("Failed: {}", e)),
    }
}

fn clipboard_read(_args: &Value) -> ToolResult {
    match Command::new("pbpaste").output() {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            ToolResult::ok(text.to_string())
        }
        Ok(out) => ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => ToolResult::err(format!("Failed to read clipboard: {}", e)),
    }
}

fn clipboard_write(args: &Value) -> ToolResult {
    let text = args["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return ToolResult::err("No text provided");
    }
    let mut child = match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => return ToolResult::err(format!("Failed to run pbcopy: {}", e)),
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
    }
    match child.wait() {
        Ok(status) if status.success() => ToolResult::ok("Copied to clipboard"),
        _ => ToolResult::err("Failed to copy to clipboard"),
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn osascript(script: &str) -> Result<String, String> {
    match Command::new("osascript").arg("-e").arg(script).output() {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).trim().to_string()),
        Err(e) => Err(format!("osascript failed: {}", e)),
    }
}

fn send_result(stdout: &io::Stdout, id: Option<&Value>, result: Value) {
    let mut msg = json!({ "jsonrpc": "2.0", "result": result });
    if let Some(id) = id {
        msg["id"] = id.clone();
    }
    send_json(stdout, msg);
}

fn send_error(stdout: &io::Stdout, id: &Value, code: i32, message: String) {
    send_json(
        stdout,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message }
        }),
    );
}

fn send_json(stdout: &io::Stdout, value: Value) {
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{}", value);
    let _ = handle.flush();
}
