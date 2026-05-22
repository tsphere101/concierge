mod handlers;
mod tool;
mod transport;

use std::io::{self, BufRead};

use serde_json::{json, Value};

use crate::tool::{handle_tool, tools};
use crate::transport::{send_error, send_result};

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = msg["method"].as_str().unwrap_or("");
        let id = msg.get("id");

        match method {
            "initialize" => {
                send_result(
                    &stdout,
                    id,
                    json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": {} },
                        "serverInfo": { "name": "concierge", "version": "0.1.0" }
                    }),
                );
            }
            "notifications/initialized" => {}
            "tools/list" => {
                let tools: Vec<Value> = tools()
                    .into_iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": t.schema
                        })
                    })
                    .collect();
                send_result(&stdout, id, json!({ "tools": tools }));
            }
            "tools/call" => {
                let name = msg["params"]["name"].as_str().unwrap_or("");
                let args = &msg["params"]["arguments"];
                let result = handle_tool(name, args);
                let mut response = json!({
                    "content": [{ "type": "text", "text": result.text }]
                });
                if result.is_error {
                    response["isError"] = json!(true);
                }
                send_result(&stdout, id, response);
            }
            _ => {
                if let Some(id) = id {
                    send_error(
                        &stdout,
                        id,
                        -32601,
                        format!("Method not found: {method}"),
                    );
                }
            }
        }
    }
}
