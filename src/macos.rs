use std::path::Path;

use crate::cmd::CmdRunner;

pub fn osascript(runner: &dyn CmdRunner, script: &str) -> Result<String, String> {
    runner.output_str("osascript", &["-e", script])
}

pub fn battery(runner: &dyn CmdRunner) -> Result<String, String> {
    runner.output_str("pmset", &["-g", "batt"])
}

pub fn clipboard_read(runner: &dyn CmdRunner) -> Result<String, String> {
    runner.output_str("pbpaste", &[])
}

pub fn clipboard_write(runner: &dyn CmdRunner, text: &str) -> Result<(), String> {
    runner.output_with_stdin_str("pbcopy", &[], text)
}

pub fn open(runner: &dyn CmdRunner, target: &str) -> Result<(), String> {
    runner.output_str("open", &[target]).map(|_| ())
}

pub fn reveal(runner: &dyn CmdRunner, path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("Path not found: {path}"));
    }
    runner.output_str("open", &["-R", path]).map(|_| ())
}

pub fn volume_get(runner: &dyn CmdRunner) -> Result<String, String> {
    osascript(runner, "output volume of (get volume settings)")
}

pub fn volume_set(runner: &dyn CmdRunner, level: i64) -> Result<(), String> {
    let level = level.clamp(0, 100);
    osascript(runner, &format!("set volume output volume {level}"))?;
    Ok(())
}

pub fn volume_mute(runner: &dyn CmdRunner) -> Result<(), String> {
    osascript(runner, "set volume with output muted")?;
    Ok(())
}

pub fn volume_unmute(runner: &dyn CmdRunner) -> Result<(), String> {
    osascript(runner, "set volume without output muted")?;
    Ok(())
}

pub fn say(runner: &dyn CmdRunner, text: &str, voice: Option<&str>, rate: Option<f64>) -> Result<(), String> {
    let mut args: Vec<String> = Vec::new();
    if let Some(v) = voice {
        args.push("-v".into());
        args.push(v.to_string());
    }
    if let Some(r) = rate {
        args.push("-r".into());
        args.push(r.to_string());
    }
    args.push(text.to_string());
    runner.spawn("say", &args).map(|_| ())
}

pub fn brew_service(runner: &dyn CmdRunner, action: &str, service: &str) -> Result<String, String> {
    runner.output_str("brew", &["services", action, service])
}

pub fn rest(runner: &dyn CmdRunner) -> Result<(), String> {
    let script = r#"tell application "Amphetamine" to end session"#;
    osascript(runner, script)?;
    Ok(())
}

pub fn wake(runner: &dyn CmdRunner, hours: u64) -> Result<(), String> {
    let script = format!(
        r#"tell application "Amphetamine" to start new session with options {{duration:{hours}, interval:hours, displaySleepAllowed:false}}"#
    );
    osascript(runner, &script)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::MockCmdRunner;

    #[test]
    fn test_battery_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("99%"));
        assert_eq!(battery(&mock), Ok("99%".into()));
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output pmset -g batt"
        );
    }

    #[test]
    fn test_battery_err() {
        let mock = MockCmdRunner::new()
            .output_result(Err("no battery"));
        assert_eq!(battery(&mock), Err("no battery".into()));
    }

    #[test]
    fn test_osascript_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("42"));
        assert_eq!(osascript(&mock, "test script"), Ok("42".into()));
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output osascript -e test script"
        );
    }

    #[test]
    fn test_clipboard_read_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("hello"));
        assert_eq!(clipboard_read(&mock), Ok("hello".into()));
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output pbpaste "
        );
    }

    #[test]
    fn test_clipboard_write_ok() {
        let mock = MockCmdRunner::new()
            .stdin_result(Ok(()));
        assert!(clipboard_write(&mock, "data").is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "stdin pbcopy  data=data"
        );
    }

    #[test]
    fn test_clipboard_write_err() {
        let mock = MockCmdRunner::new()
            .stdin_result(Err("permission denied"));
        assert_eq!(clipboard_write(&mock, "data"), Err("permission denied".into()));
    }

    #[test]
    fn test_open_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(open(&mock, "/tmp").is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output open /tmp"
        );
    }

    #[test]
    fn test_open_err() {
        let mock = MockCmdRunner::new()
            .output_result(Err("not found"));
        assert_eq!(open(&mock, "/nonexistent"), Err("not found".into()));
    }

    #[test]
    fn test_reveal_path_not_found() {
        let mock = MockCmdRunner::new();
        let result = reveal(&mock, "/nonexistent_path_xyz_abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Path not found"));
    }

    #[test]
    fn test_volume_get_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("50"));
        assert_eq!(volume_get(&mock), Ok("50".into()));
    }

    #[test]
    fn test_volume_set_clamps() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(volume_set(&mock, 150).is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output osascript -e set volume output volume 100"
        );
    }

    #[test]
    fn test_volume_set_negative_clamps() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(volume_set(&mock, -10).is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output osascript -e set volume output volume 0"
        );
    }

    #[test]
    fn test_volume_mute() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(volume_mute(&mock).is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output osascript -e set volume with output muted"
        );
    }

    #[test]
    fn test_volume_unmute() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(volume_unmute(&mock).is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output osascript -e set volume without output muted"
        );
    }

    #[test]
    fn test_say_default() {
        let mock = MockCmdRunner::new()
            .spawn_result(Ok(()));
        assert!(say(&mock, "hello", None, None).is_ok());
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "spawn say hello"
        );
    }

    #[test]
    fn test_say_with_voice_and_rate() {
        let mock = MockCmdRunner::new()
            .spawn_result(Ok(()));
        assert!(say(&mock, "hello", Some("Samantha"), Some(200.0)).is_ok());
        let calls = mock.calls.lock().unwrap();
        assert!(calls[0].contains("spawn say -v Samantha -r 200 hello"));
    }

    #[test]
    fn test_say_err() {
        let mock = MockCmdRunner::new()
            .spawn_result(Err("failed"));
        assert_eq!(say(&mock, "hello", None, None), Err("failed".into()));
    }

    #[test]
    fn test_brew_service_start_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("started"));
        assert_eq!(brew_service(&mock, "start", "nginx"), Ok("started".into()));
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output brew services start nginx"
        );
    }

    #[test]
    fn test_brew_service_stop_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("stopped"));
        assert_eq!(brew_service(&mock, "stop", "nginx"), Ok("stopped".into()));
        assert_eq!(
            mock.calls.lock().unwrap()[0],
            "output brew services stop nginx"
        );
    }

    #[test]
    fn test_brew_service_err() {
        let mock = MockCmdRunner::new()
            .output_result(Err("brew not found"));
        assert_eq!(brew_service(&mock, "start", "nginx"), Err("brew not found".into()));
    }

    #[test]
    fn test_rest_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(rest(&mock).is_ok());
        let calls = mock.calls.lock().unwrap();
        assert!(calls[0].contains("end session"));
    }

    #[test]
    fn test_rest_err() {
        let mock = MockCmdRunner::new()
            .output_result(Err("Amphetamine not found"));
        assert_eq!(rest(&mock), Err("Amphetamine not found".into()));
    }

    #[test]
    fn test_wake_ok() {
        let mock = MockCmdRunner::new()
            .output_result(Ok(""));
        assert!(wake(&mock, 2).is_ok());
        let calls = mock.calls.lock().unwrap();
        assert!(calls[0].contains("osascript"));
        assert!(calls[0].contains("duration:2"));
    }

    #[test]
    fn test_wake_err() {
        let mock = MockCmdRunner::new()
            .output_result(Err("Amphetamine not found"));
        assert_eq!(wake(&mock, 1), Err("Amphetamine not found".into()));
    }
}
