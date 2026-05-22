use serde_json::{json, Value};
use std::io::Write;
use std::process::Command;

pub fn osascript(script: &str) -> Result<String, String> {
    match Command::new("osascript").arg("-e").arg(script).output() {
        Ok(out) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).trim().to_string()),
        Err(e) => Err(format!("osascript failed: {}", e)),
    }
}

pub fn send_result(stdout: &std::io::Stdout, id: Option<&Value>, result: Value) {
    let mut msg = json!({ "jsonrpc": "2.0", "result": result });
    if let Some(id) = id {
        msg["id"] = id.clone();
    }
    send_json(stdout, msg);
}

pub fn send_error(stdout: &std::io::Stdout, id: &Value, code: i32, message: String) {
    send_json(
        stdout,
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message }
        }),
    );
}

pub fn send_json(stdout: &std::io::Stdout, value: Value) {
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{value}");
    let _ = handle.flush();
}
