use serde_json::{json, Value};

use crate::handlers;

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
    handler: fn(&Value) -> ToolResult,
}

pub struct ToolResult {
    pub text: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn ok(text: impl Into<String>) -> Self {
        ToolResult {
            text: text.into(),
            is_error: false,
        }
    }

    pub fn err(text: impl Into<String>) -> Self {
        ToolResult {
            text: text.into(),
            is_error: true,
        }
    }
}

pub fn tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "battery",
            description: "Get battery status",
            schema: json!({ "type": "object", "properties": {} }),
            handler: handlers::battery,
        },
        ToolDef {
            name: "clipboard-read",
            description: "Read text from the clipboard",
            schema: json!({ "type": "object", "properties": {} }),
            handler: handlers::clipboard_read,
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
            handler: handlers::clipboard_write,
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
            handler: handlers::open_tool,
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
            handler: handlers::reveal,
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
            handler: handlers::say_tool,
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
            handler: handlers::volume,
        },
        ToolDef {
            name: "wake",
            description: "Keep the Mac awake for N hours using Amphetamine",
            schema: json!({
                "type": "object",
                "properties": {
                    "hours": { "type": "number", "description": "Number of hours (default 1)" }
                }
            }),
            handler: handlers::wake,
        },
    ]
}

pub fn handle_tool(name: &str, args: &Value) -> ToolResult {
    for tool in tools() {
        if tool.name == name {
            return (tool.handler)(args);
        }
    }
    ToolResult::err(format!("Unknown tool: {name}"))
}
