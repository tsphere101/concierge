const DISCORD_MAX_CONTENT_LENGTH: usize = 2000;

pub fn send_discord_message(content: &str) -> Result<(), String> {
    if content.len() > DISCORD_MAX_CONTENT_LENGTH {
        return Err(format!(
            "Message exceeds Discord's 2000 character limit (current: {} chars)",
            content.len()
        ));
    }

    let url = std::env::var("DISCORD_WEBHOOK_URL")
        .map_err(|_| "DISCORD_WEBHOOK_URL environment variable not set".to_string())?;

    let body = serde_json::json!({ "content": content });

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("Failed to send Discord webhook: {e}"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(handle_discord_error(resp))
    }
}

fn handle_discord_error(resp: reqwest::blocking::Response) -> String {
    let status = resp.status();
    let body: serde_json::Value = resp.json().unwrap_or_default();
    let msg = body["message"].as_str().unwrap_or("Unknown error");

    match status.as_u16() {
        400 => format!("Discord: Bad request — {msg}"),
        401 | 403 => format!("Discord: Unauthorized — {msg}"),
        404 => format!("Discord: Webhook not found — {msg}"),
        429 => {
            let retry = body["retry_after"].as_f64().unwrap_or(0.0);
            format!("Discord: Rate limited — retry after {retry:.0}s")
        }
        _ => format!("Discord error {status}: {msg}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_exceeds_limit() {
        let long_msg = "x".repeat(2001);
        let result = send_discord_message(&long_msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("2000 character limit"));
    }

    #[test]
    fn test_content_at_limit_is_allowed_before_url_check() {
        let msg = "x".repeat(2000);
        let result = send_discord_message(&msg);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("DISCORD_WEBHOOK_URL"));
    }

    #[test]
    fn test_missing_env_var() {
        std::env::remove_var("DISCORD_WEBHOOK_URL");
        let result = send_discord_message("hello");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("DISCORD_WEBHOOK_URL"));
    }

    #[test]
    fn test_handle_discord_error_rate_limit() {
        let raw = r#"{"message": "You are being rate limited.", "retry_after": 12.34}"#;
        let resp = httpmock_response(429, raw);
        let err = handle_discord_error(resp);
        assert!(err.contains("Rate limited"));
        assert!(err.contains("12s"));
    }

    #[test]
    fn test_handle_discord_error_unauthorized() {
        let raw = r#"{"message": "Invalid Webhook Token", "code": 50027}"#;
        let resp = httpmock_response(401, raw);
        let err = handle_discord_error(resp);
        assert!(err.contains("Unauthorized"));
        assert!(err.contains("Invalid Webhook Token"));
    }

    #[test]
    fn test_handle_discord_error_not_found() {
        let raw = r#"{"message": "Unknown Webhook", "code": 10015}"#;
        let resp = httpmock_response(404, raw);
        let err = handle_discord_error(resp);
        assert!(err.contains("Webhook not found"));
    }

    #[test]
    fn test_handle_discord_error_bad_request() {
        let raw = r#"{"message": "Cannot send an empty message", "code": 50006}"#;
        let resp = httpmock_response(400, raw);
        let err = handle_discord_error(resp);
        assert!(err.contains("Bad request"));
    }

    fn httpmock_response(status: u16, body_json: &str) -> reqwest::blocking::Response {
        reqwest::blocking::Response::from(
            http::response::Builder::new()
                .status(status)
                .body(body_json.as_bytes().to_vec())
                .unwrap(),
        )
    }
}
