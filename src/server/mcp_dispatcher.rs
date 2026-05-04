use serde_json::{json, Value};

use crate::server::mcp_schemas;
use crate::server::mcp_handler::SyntagmaMCP;

/// Dispatch a single JSON-RPC 2.0 request against the MCP handler.
///
/// Returns `None` for notifications (no response expected).
/// Returns `Some(response)` for requests that expect a response.
pub fn dispatch(mcp: &SyntagmaMCP, request: Value) -> Option<Value> {
    let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");
    let params = request.get("params").cloned().unwrap_or(json!({}));
    let req_id = request.get("id").cloned();

    match method {
        // -- protocol lifecycle ------------------------------------------------
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": mcp_schemas::PROTOCOL_VERSION,
                "capabilities": mcp_schemas::capabilities(),
                "serverInfo": mcp_schemas::server_info(),
                "instructions": mcp_schemas::server_instructions(),
            },
        })),

        "notifications/initialized" => None,

        "ping" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {},
        })),

        // -- tools -------------------------------------------------------------
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": mcp_schemas::tool_schemas(),
            },
        })),

        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            let result = mcp.handle_tool_call(tool_name, &arguments);
            Some(tool_text_response(req_id, result))
        }

        // -- resources ---------------------------------------------------------
        "resources/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "resources": mcp_schemas::resource_schemas(),
            },
        })),

        "resources/read" => {
            let uri = params
                .get("uri")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let data = mcp.handle_resource_read(uri);
            Some(json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": serde_json::to_string(&data).unwrap_or_default(),
                    }]
                },
            }))
        }

        // -- unknown -----------------------------------------------------------
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {
                "code": -32601,
                "message": "Method not found",
            },
        })),
    }
}

/// Wrap a result value as an MCP text content block within a JSON-RPC response.
fn tool_text_response(req_id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "content": [{
                "type": "text",
                "text": serde_json::to_string(&result).unwrap_or_default(),
            }]
        },
    })
}
