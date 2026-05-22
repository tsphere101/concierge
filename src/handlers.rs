use std::path::Path;
use std::process::{Command, Stdio};

use crate::tool::ToolResult;
use crate::transport::osascript;

pub fn wake(args: &serde_json::Value) -> ToolResult {
    let hours = args["hours"].as_f64().unwrap_or(1.0) as u64;
    let script = format!(
        r#"tell application "Amphetamine" to start new session with options {{duration:{hours}, interval:hours, displaySleepAllowed:false}}"#,
    );
    match osascript(&script) {
        Ok(_) => ToolResult::ok(format!("Amphetamine session started for {hours} hour(s)")),
        Err(e) => ToolResult::err(e),
    }
}

pub fn open_tool(args: &serde_json::Value) -> ToolResult {
    let target = args["target"].as_str().unwrap_or("");
    if target.is_empty() {
        return ToolResult::err("No target provided");
    }
    match Command::new("open").arg(target).output() {
        Ok(out) if out.status.success() => ToolResult::ok(format!("Opened: {target}")),
        Ok(out) => ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => ToolResult::err(format!("Error: {e}")),
    }
}

pub fn reveal(args: &serde_json::Value) -> ToolResult {
    let path = args["path"].as_str().unwrap_or("");
    if path.is_empty() {
        return ToolResult::err("No path provided");
    }
    if !Path::new(path).exists() {
        return ToolResult::err(format!("Path not found: {path}"));
    }
    match Command::new("open").arg("-R").arg(path).output() {
        Ok(out) if out.status.success() => ToolResult::ok(format!("Revealed: {path}")),
        Ok(out) => ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => ToolResult::err(format!("Error: {e}")),
    }
}

pub fn volume(args: &serde_json::Value) -> ToolResult {
    let action = args["action"].as_str().unwrap_or("get");
    match action {
        "set" => {
            let level = args["level"].as_f64().unwrap_or(50.0) as i64;
            let level = level.clamp(0, 100);
            match osascript(&format!("set volume output volume {level}")) {
                Ok(_) => ToolResult::ok(format!("Volume set to {level}%")),
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
            "Unknown action: {action}. Use set/get/mute/unmute"
        )),
    }
}

pub fn battery(_args: &serde_json::Value) -> ToolResult {
    match Command::new("pmset").arg("-g").arg("batt").output() {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            ToolResult::ok(text.trim().to_string())
        }
        Err(e) => ToolResult::err(format!("Failed to get battery status: {e}")),
    }
}

pub fn say_tool(args: &serde_json::Value) -> ToolResult {
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
        Ok(_) => ToolResult::ok(format!("Speaking: {text}")),
        Err(e) => ToolResult::err(format!("Failed: {e}")),
    }
}

pub fn clipboard_read(_args: &serde_json::Value) -> ToolResult {
    match Command::new("pbpaste").output() {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            ToolResult::ok(text.to_string())
        }
        Ok(out) => ToolResult::err(String::from_utf8_lossy(&out.stderr).to_string()),
        Err(e) => ToolResult::err(format!("Failed to read clipboard: {e}")),
    }
}

pub fn clipboard_write(args: &serde_json::Value) -> ToolResult {
    let text = args["text"].as_str().unwrap_or("");
    if text.is_empty() {
        return ToolResult::err("No text provided");
    }
    let mut child = match Command::new("pbcopy").stdin(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) => return ToolResult::err(format!("Failed to run pbcopy: {e}")),
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = std::io::Write::write_all(&mut stdin, text.as_bytes());
    }
    match child.wait() {
        Ok(status) if status.success() => ToolResult::ok("Copied to clipboard"),
        _ => ToolResult::err("Failed to copy to clipboard"),
    }
}
