mod args;
mod cmd;
mod macos;
mod webhook;

use std::sync::Arc;

use args::{BrewServiceArgs, ClipboardWriteArgs, DiscordWebhookArgs, OpenArgs, RevealArgs, SayArgs, VolumeArgs, WakeArgs};
use cmd::CmdRunner;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::*,
    prompt, prompt_handler, prompt_router,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};

// ── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Concierge {
    runner: Arc<dyn CmdRunner>,
}

fn result_to_string<T>(result: Result<T, String>, ok_msg: impl FnOnce(T) -> String) -> String {
    match result {
        Ok(v) => ok_msg(v),
        Err(e) => format!("Error: {e}"),
    }
}

// ── Tools ───────────────────────────────────────────────────────────────────

#[tool_router]
impl Concierge {
    #[tool(
        title = "Battery",
        description = "Get battery status",
        annotations(read_only_hint = true)
    )]
    async fn battery(&self) -> String {
        result_to_string(macos::battery(&*self.runner), |s| s)
    }

    #[tool(
        title = "Clipboard Read",
        description = "Read text from the clipboard",
        annotations(read_only_hint = true)
    )]
    async fn clipboard_read(&self) -> String {
        result_to_string(macos::clipboard_read(&*self.runner), |s| s)
    }

    #[tool(
        title = "Clipboard Write",
        description = "Write text to the clipboard",
        annotations(destructive_hint = false)
    )]
    async fn clipboard_write(&self, Parameters(args): Parameters<ClipboardWriteArgs>) -> String {
        result_to_string(macos::clipboard_write(&*self.runner, &args.text), |_| "Copied to clipboard".into())
    }

    #[tool(
        title = "Open",
        description = "Open a file path or URL with the default application"
    )]
    async fn open(&self, Parameters(args): Parameters<OpenArgs>) -> String {
        result_to_string(macos::open(&*self.runner, &args.target), |_| format!("Opened: {}", args.target))
    }

    #[tool(
        title = "Reveal in Finder",
        description = "Reveal a file or folder in Finder"
    )]
    async fn reveal(&self, Parameters(args): Parameters<RevealArgs>) -> String {
        result_to_string(macos::reveal(&*self.runner, &args.path), |_| format!("Revealed: {}", args.path))
    }

    #[tool(
        title = "Volume",
        description = "Control system output volume"
    )]
    async fn volume(&self, Parameters(args): Parameters<VolumeArgs>) -> String {
        let runner = &*self.runner;
        match args.action.as_str() {
            "set" => {
                let level = args.level.unwrap_or(50.0) as i64;
                result_to_string(macos::volume_set(runner, level), |_| format!("Volume set to {level}%"))
            }
            "get" => result_to_string(macos::volume_get(runner), |v| format!("Volume: {}%", v.trim())),
            "mute" => result_to_string(macos::volume_mute(runner), |_| "Muted".into()),
            "unmute" => result_to_string(macos::volume_unmute(runner), |_| "Unmuted".into()),
            _ => format!("Unknown action: {}. Use set/get/mute/unmute", args.action),
        }
    }

    #[tool(
        title = "Text-to-Speech",
        description = "Speak text aloud using text-to-speech"
    )]
    async fn say(&self, Parameters(args): Parameters<SayArgs>) -> String {
        result_to_string(
            macos::say(&*self.runner, &args.text, args.voice.as_deref(), args.rate),
            |_| format!("Speaking: {}", args.text),
        )
    }

    #[tool(
        title = "Wake Status",
        description = "Check current Amphetamine wake prevention session status",
        annotations(read_only_hint = true)
    )]
    async fn wake_status(&self) -> String {
        result_to_string(macos::amphetamine_status(&*self.runner), |s| s)
    }

    #[tool(
        title = "Rest",
        description = "End Amphetamine wake prevention session"
    )]
    async fn rest(&self) -> String {
        result_to_string(macos::rest(&*self.runner), |_| "Amphetamine session ended".into())
    }

    #[tool(
        title = "Wake",
        description = "Keep the Mac awake for N hours using Amphetamine"
    )]
    async fn wake(&self, Parameters(args): Parameters<WakeArgs>) -> String {
        let hours = args.hours.unwrap_or(1.0) as u64;
        result_to_string(macos::wake(&*self.runner, hours), |_| format!("Amphetamine session started for {hours} hour(s)"))
    }

    #[tool(
        title = "Brew Service",
        description = "Start or stop a Homebrew service (e.g. nginx)"
    )]
    async fn brew_service(&self, Parameters(args): Parameters<BrewServiceArgs>) -> String {
        let service = args.service.as_deref().unwrap_or("nginx");
        match args.action.as_str() {
            "start" => result_to_string(
                macos::brew_service(&*self.runner, "start", service),
                |_| format!("Started {service}"),
            ),
            "stop" => result_to_string(
                macos::brew_service(&*self.runner, "stop", service),
                |_| format!("Stopped {service}"),
            ),
            _ => format!("Unknown action: {}. Use start or stop", args.action),
        }
    }

    #[tool(
        title = "Discord Webhook",
        description = "Send a message to Discord via webhook"
    )]
    async fn discord_webhook(&self, Parameters(args): Parameters<DiscordWebhookArgs>) -> String {
        result_to_string(
            webhook::send_discord_message(&args.content),
            |_| "Message sent to Discord".into(),
        )
    }
}

// ── Prompts ─────────────────────────────────────────────────────────────────

#[prompt_router]
impl Concierge {
    #[prompt(
        name = "system-status",
        description = "Get a full system status report (battery, volume, clipboard)"
    )]
    async fn system_status(&self) -> Vec<PromptMessage> {
        let runner = &*self.runner;
        let battery = macos::battery(runner).unwrap_or_else(|e| format!("(unavailable: {e})"));
        let volume = macos::volume_get(runner).unwrap_or_else(|e| format!("(unavailable: {e})"));
        let clipboard = macos::clipboard_read(runner).unwrap_or_else(|e| format!("(unavailable: {e})"));

        vec![PromptMessage::new_text(
            PromptMessageRole::User,
            format!(
                "Give me a concise system status report.\n\n\
                 Battery: {battery}\n\
                 Volume: {volume}\n\
                 Clipboard contents: {clipboard}\n\n\
                 Summarize the key information.",
            ),
        )]
    }
}

// ── ServerHandler ───────────────────────────────────────────────────────────

#[tool_handler]
#[prompt_handler]
impl ServerHandler for Concierge {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .enable_logging()
                .build(),
        );
        info.server_info = Implementation::new("concierge", "0.2.0");
        info.instructions = Some(
            "macOS system control tools — battery, clipboard, volume, \
             text-to-speech, file operations, and wake prevention."
                .into(),
        );
        info
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource {
                    uri: "concierge://system/battery".into(),
                    name: "Battery Status".into(),
                    title: None,
                    description: Some(
                        "Current battery level, charging status, and time remaining".into(),
                    ),
                    mime_type: None,
                    size: None,
                    icons: None,
                    meta: None,
                }
                .no_annotation(),
                RawResource {
                    uri: "concierge://system/volume".into(),
                    name: "Volume Level".into(),
                    title: None,
                    description: Some(
                        "Current system output volume level and mute state".into(),
                    ),
                    mime_type: None,
                    size: None,
                    icons: None,
                    meta: None,
                }
                .no_annotation(),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let text = match request.uri.as_str() {
            "concierge://system/battery" => macos::battery(&*self.runner),
            "concierge://system/volume" => macos::volume_get(&*self.runner),
            _ => {
                return Err(McpError::resource_not_found(
                    "resource_not_found",
                    Some(serde_json::json!({ "uri": request.uri })),
                ))
            }
        };

        let text = text.map_err(|e| McpError::internal_error(e, None))?;

        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            text,
            &request.uri,
        )]))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![],
            next_cursor: None,
            meta: None,
        })
    }

    async fn set_level(
        &self,
        _request: SetLevelRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        Ok(())
    }
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let concierge = Concierge {
        runner: Arc::new(cmd::RealCmdRunner),
    };
    let service = concierge.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_to_string_ok() {
        assert_eq!(
            result_to_string::<&str>(Ok("hello"), |s| format!("got: {s}")),
            "got: hello"
        );
    }

    #[test]
    fn test_result_to_string_err() {
        assert_eq!(
            result_to_string::<()>(Err("failed".into()), |_| unreachable!()),
            "Error: failed"
        );
    }
}
