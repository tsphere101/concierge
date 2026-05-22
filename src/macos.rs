use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn osascript(script: &str) -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript failed: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn battery() -> Result<String, String> {
    let output = Command::new("pmset")
        .arg("-g")
        .arg("batt")
        .output()
        .map_err(|e| format!("Failed to get battery status: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn clipboard_read() -> Result<String, String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| format!("Failed to read clipboard: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn clipboard_write(text: &str) -> Result<(), String> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to run pbcopy: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("Failed to write to pbcopy: {e}"))?;
    }

    child
        .wait()
        .map_err(|e| format!("Failed to wait for pbcopy: {e}"))?;

    Ok(())
}

pub fn open(target: &str) -> Result<(), String> {
    let output = Command::new("open")
        .arg(target)
        .output()
        .map_err(|e| format!("Error: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn reveal(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("Path not found: {path}"));
    }

    let output = Command::new("open")
        .arg("-R")
        .arg(path)
        .output()
        .map_err(|e| format!("Error: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn volume_get() -> Result<String, String> {
    osascript("output volume of (get volume settings)")
}

pub fn volume_set(level: i64) -> Result<(), String> {
    let level = level.clamp(0, 100);
    osascript(&format!("set volume output volume {level}"))?;
    Ok(())
}

pub fn volume_mute() -> Result<(), String> {
    osascript("set volume with output muted")?;
    Ok(())
}

pub fn volume_unmute() -> Result<(), String> {
    osascript("set volume without output muted")?;
    Ok(())
}

pub fn say(text: &str, voice: Option<&str>, rate: Option<f64>) -> Result<(), String> {
    let mut cmd = Command::new("say");
    if let Some(v) = voice {
        cmd.arg("-v").arg(v);
    }
    if let Some(r) = rate {
        cmd.arg("-r").arg(r.to_string());
    }
    cmd.arg(text);

    cmd.spawn().map_err(|e| format!("Failed: {e}"))?;

    Ok(())
}

pub fn wake(hours: u64) -> Result<(), String> {
    let script = format!(
        r#"tell application "Amphetamine" to start new session with options {{duration:{hours}, interval:hours, displaySleepAllowed:false}}"#
    );
    osascript(&script)?;
    Ok(())
}
