use std::io::Write;
use std::process::{Command, Stdio};

pub trait CmdRunner: Send + Sync {
    fn output(&self, program: &str, args: &[String]) -> Result<String, String>;

    fn output_str(&self, program: &str, args: &[&str]) -> Result<String, String> {
        self.output(program, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    fn output_with_stdin(&self, program: &str, args: &[String], data: &str) -> Result<(), String>;

    fn output_with_stdin_str(
        &self,
        program: &str,
        args: &[&str],
        data: &str,
    ) -> Result<(), String> {
        self.output_with_stdin(program, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(), data)
    }

    fn spawn(&self, program: &str, args: &[String]) -> Result<(), String>;

    fn spawn_str(&self, program: &str, args: &[&str]) -> Result<(), String> {
        self.spawn(program, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }
}

#[derive(Clone)]
pub struct RealCmdRunner;

impl CmdRunner for RealCmdRunner {
    fn output(&self, program: &str, args: &[String]) -> Result<String, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute {program}: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
        }
    }

    fn output_with_stdin(&self, program: &str, args: &[String], data: &str) -> Result<(), String> {
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to run {program}: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data.as_bytes())
                .map_err(|e| format!("Failed to write to {program}: {e}"))?;
        }

        child
            .wait()
            .map_err(|e| format!("Failed to wait for {program}: {e}"))?;

        Ok(())
    }

    fn spawn(&self, program: &str, args: &[String]) -> Result<(), String> {
        Command::new(program)
            .args(args)
            .spawn()
            .map_err(|e| format!("Failed to spawn {program}: {e}"))?;
        Ok(())
    }
}

use std::sync::Mutex;
use std::collections::VecDeque;

pub struct MockCmdRunner {
    output_results: Mutex<VecDeque<Result<String, String>>>,
    stdin_results: Mutex<VecDeque<Result<(), String>>>,
    spawn_results: Mutex<VecDeque<Result<(), String>>>,
    pub calls: Mutex<Vec<String>>,
}

impl MockCmdRunner {
    pub fn new() -> Self {
        Self {
            output_results: Mutex::new(VecDeque::new()),
            stdin_results: Mutex::new(VecDeque::new()),
            spawn_results: Mutex::new(VecDeque::new()),
            calls: Mutex::new(Vec::new()),
        }
    }

    pub fn output_result(self, result: Result<&str, &str>) -> Self {
        self.output_results.lock().unwrap().push_back(result.map(|s| s.to_string()).map_err(|s| s.to_string()));
        self
    }

    pub fn stdin_result(self, result: Result<(), &str>) -> Self {
        self.stdin_results.lock().unwrap().push_back(result.map_err(|s| s.to_string()));
        self
    }

    pub fn spawn_result(self, result: Result<(), &str>) -> Self {
        self.spawn_results.lock().unwrap().push_back(result.map_err(|s| s.to_string()));
        self
    }
}

impl Default for MockCmdRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CmdRunner for MockCmdRunner {
    fn output(&self, program: &str, args: &[String]) -> Result<String, String> {
        self.calls.lock().unwrap().push(format!("output {} {}", program, args.join(" ")));
        let mut queue = self.output_results.lock().unwrap();
        queue.pop_front().unwrap_or(Ok(String::new()))
    }

    fn output_with_stdin(&self, program: &str, args: &[String], data: &str) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("stdin {} {} data={}", program, args.join(" "), data));
        let mut queue = self.stdin_results.lock().unwrap();
        queue.pop_front().unwrap_or(Ok(()))
    }

    fn spawn(&self, program: &str, args: &[String]) -> Result<(), String> {
        self.calls.lock().unwrap().push(format!("spawn {} {}", program, args.join(" ")));
        let mut queue = self.spawn_results.lock().unwrap();
        queue.pop_front().unwrap_or(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_records_output_call() {
        let mock = MockCmdRunner::new()
            .output_result(Ok("test output"));
        let result = mock.output_str("echo", &["hello"]);
        assert_eq!(result, Ok("test output".into()));
        let calls = mock.calls.lock().unwrap();
        assert_eq!(calls[0], "output echo hello");
    }

    #[test]
    fn test_mock_records_stdin_call() {
        let mock = MockCmdRunner::new()
            .stdin_result(Ok(()));
        let result = mock.output_with_stdin_str("pbcopy", &[], "data");
        assert!(result.is_ok());
        let calls = mock.calls.lock().unwrap();
        assert_eq!(calls[0], "stdin pbcopy  data=data");
    }

    #[test]
    fn test_mock_records_spawn_call() {
        let mock = MockCmdRunner::new()
            .spawn_result(Ok(()));
        let result = mock.spawn_str("say", &["hello"]);
        assert!(result.is_ok());
        let calls = mock.calls.lock().unwrap();
        assert_eq!(calls[0], "spawn say hello");
    }

    #[test]
    fn test_real_output_errors_on_bad_program() {
        let runner = RealCmdRunner;
        let result = runner.output("nonexistent_cmd_xyz", &[]);
        assert!(result.is_err());
    }
}
