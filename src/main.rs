mod macos;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::*,
    prompt, prompt_handler, prompt_router,
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;

// ── Tool argument structs ───────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ClipboardWriteArgs {
    text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct OpenArgs {
    target: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RevealArgs {
    path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct VolumeArgs {
    action: String,
    level: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SayArgs {
    text: String,
    voice: Option<String>,
    rate: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct WakeArgs {
    hours: Option<f64>,
}

// ── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Concierge;

// ── Tools ───────────────────────────────────────────────────────────────────

#[tool_router]
impl Concierge {
    #[tool(
        title = "Battery",
        description = "Get battery status",
        annotations(read_only_hint = true)
    )]
    async fn battery(&self) -> String {
        match macos::battery() {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        title = "Clipboard Read",
        description = "Read text from the clipboard",
        annotations(read_only_hint = true)
    )]
    async fn clipboard_read(&self) -> String {
        match macos::clipboard_read() {
            Ok(s) => s,
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        title = "Clipboard Write",
        description = "Write text to the clipboard",
        annotations(destructive_hint = false)
    )]
    async fn clipboard_write(&self, Parameters(args): Parameters<ClipboardWriteArgs>) -> String {
        match macos::clipboard_write(&args.text) {
            Ok(_) => "Copied to clipboard".into(),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        title = "Open",
        description = "Open a file path or URL with the default application"
    )]
    async fn open(&self, Parameters(args): Parameters<OpenArgs>) -> String {
        match macos::open(&args.target) {
            Ok(_) => format!("Opened: {}", args.target),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        title = "Reveal in Finder",
        description = "Reveal a file or folder in Finder"
    )]
    async fn reveal(&self, Parameters(args): Parameters<RevealArgs>) -> String {
        match macos::reveal(&args.path) {
            Ok(_) => format!("Revealed: {}", args.path),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        title = "Volume",
        description = "Control system output volume"
    )]
    async fn volume(&self, Parameters(args): Parameters<VolumeArgs>) -> String {
        match args.action.as_str() {
            "set" => {
                let level = args.level.unwrap_or(50.0) as i64;
                match macos::volume_set(level) {
                    Ok(_) => format!("Volume set to {level}%"),
                    Err(e) => format!("Error: {e}"),
                }
            }
            "get" => match macos::volume_get() {
                Ok(v) => format!("Volume: {}%", v.trim()),
                Err(e) => format!("Error: {e}"),
            },
            "mute" => match macos::volume_mute() {
                Ok(_) => "Muted".into(),
                Err(e) => format!("Error: {e}"),
            },
            "unmute" => match macos::volume_unmute() {
                Ok(_) => "Unmuted".into(),
                Err(e) => format!("Error: {e}"),
            },
            _ => format!("Unknown action: {}. Use set/get/mute/unmute", args.action),
        }
    }

    #[tool(
        title = "Text-to-Speech",
        description = "Speak text aloud using text-to-speech"
    )]
    async fn say(&self, Parameters(args): Parameters<SayArgs>) -> String {
        match macos::say(&args.text, args.voice.as_deref(), args.rate) {
            Ok(_) => format!("Speaking: {}", args.text),
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(
        title = "Wake",
        description = "Keep the Mac awake for N hours using Amphetamine"
    )]
    async fn wake(&self, Parameters(args): Parameters<WakeArgs>) -> String {
        let hours = args.hours.unwrap_or(1.0) as u64;
        match macos::wake(hours) {
            Ok(_) => format!("Amphetamine session started for {hours} hour(s)"),
            Err(e) => format!("Error: {e}"),
        }
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
        let battery = macos::battery().unwrap_or_else(|e| format!("(unavailable: {e})"));
        let volume = macos::volume_get().unwrap_or_else(|e| format!("(unavailable: {e})"));
        let clipboard = macos::clipboard_read().unwrap_or_else(|e| format!("(unavailable: {e})"));

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
            "concierge://system/battery" => macos::battery(),
            "concierge://system/volume" => macos::volume_get(),
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
    let service = Concierge.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
