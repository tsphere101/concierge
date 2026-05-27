use rmcp::schemars;
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ClipboardWriteArgs {
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct OpenArgs {
    pub target: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RevealArgs {
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct VolumeArgs {
    pub action: String,
    pub level: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SayArgs {
    pub text: String,
    pub voice: Option<String>,
    pub rate: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WakeArgs {
    pub hours: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BrewServiceArgs {
    pub action: String,
    pub service: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiscordWebhookArgs {
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_write_args_deserialize() {
        let args: ClipboardWriteArgs = serde_json::from_str(r#"{"text":"hello"}"#).unwrap();
        assert_eq!(args.text, "hello");
    }

    #[test]
    fn test_open_args_deserialize() {
        let args: OpenArgs = serde_json::from_str(r#"{"target":"/tmp"}"#).unwrap();
        assert_eq!(args.target, "/tmp");
    }

    #[test]
    fn test_reveal_args_deserialize() {
        let args: RevealArgs = serde_json::from_str(r#"{"path":"/tmp"}"#).unwrap();
        assert_eq!(args.path, "/tmp");
    }

    #[test]
    fn test_volume_args_deserialize_set() {
        let args: VolumeArgs = serde_json::from_str(r#"{"action":"set","level":75.0}"#).unwrap();
        assert_eq!(args.action, "set");
        assert_eq!(args.level, Some(75.0));
    }

    #[test]
    fn test_volume_args_deserialize_get() {
        let args: VolumeArgs = serde_json::from_str(r#"{"action":"get"}"#).unwrap();
        assert_eq!(args.action, "get");
        assert_eq!(args.level, None);
    }

    #[test]
    fn test_say_args_deserialize() {
        let args: SayArgs = serde_json::from_str(
            r#"{"text":"hello","voice":"Samantha","rate":200.0}"#,
        )
        .unwrap();
        assert_eq!(args.text, "hello");
        assert_eq!(args.voice, Some("Samantha".into()));
        assert_eq!(args.rate, Some(200.0));
    }

    #[test]
    fn test_say_args_defaults() {
        let args: SayArgs = serde_json::from_str(r#"{"text":"hello"}"#).unwrap();
        assert_eq!(args.text, "hello");
        assert_eq!(args.voice, None);
        assert_eq!(args.rate, None);
    }

    #[test]
    fn test_wake_args_deserialize() {
        let args: WakeArgs = serde_json::from_str(r#"{"hours":3.0}"#).unwrap();
        assert_eq!(args.hours, Some(3.0));
    }

    #[test]
    fn test_wake_args_defaults() {
        let args: WakeArgs = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(args.hours, None);
    }

    #[test]
    fn test_discord_webhook_args_deserialize() {
        let args: DiscordWebhookArgs =
            serde_json::from_str(r#"{"content":"hello world"}"#).unwrap();
        assert_eq!(args.content, "hello world");
    }

    #[test]
    fn test_discord_webhook_args_empty_content() {
        let args: DiscordWebhookArgs =
            serde_json::from_str(r#"{"content":""}"#).unwrap();
        assert_eq!(args.content, "");
    }
}
