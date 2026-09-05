//! End-to-end MCP test.
//!
//! Speaks newline-delimited JSON-RPC to the protocol layer exactly as a client
//! would. The socket/GUI half is exercised by hand; here we drive the message
//! handling and the tool surface, which is where the logic lives.

use serde_json::{json, Value};

/// Every advertised tool must have a name, a description and an object schema
/// — a tool the model cannot understand is worse than no tool.
#[test]
fn tool_definitions_are_well_formed() {
    let tools = sheetz::mcp::tools::definitions();
    assert!(tools.len() >= 15, "expected a useful tool surface");
    for tool in &tools {
        let name = tool["name"].as_str().expect("name");
        assert!(!name.is_empty());
        let desc = tool["description"].as_str().expect("description");
        assert!(
            desc.len() > 20,
            "{name} needs a description the model can act on"
        );
        assert_eq!(tool["inputSchema"]["type"], "object", "{name} schema");
        assert!(
            tool["inputSchema"]["properties"].is_object(),
            "{name} properties"
        );
    }
}

#[test]
fn initialize_reports_the_protocol_and_server() {
    let reply = sheetz::mcp::proto::handle_message(
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}).to_string(),
    )
    .expect("a reply");
    assert_eq!(reply["id"], 1);
    assert_eq!(
        reply["result"]["protocolVersion"],
        sheetz::mcp::proto::PROTOCOL_VERSION
    );
    assert_eq!(reply["result"]["serverInfo"]["name"], "sheetz");
    assert!(reply["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn tools_list_matches_the_definitions() {
    let reply = sheetz::mcp::proto::handle_message(
        &json!({"jsonrpc":"2.0","id":7,"method":"tools/list"}).to_string(),
    )
    .expect("a reply");
    let listed = reply["result"]["tools"].as_array().expect("tools");
    assert_eq!(listed.len(), sheetz::mcp::tools::definitions().len());
}

#[test]
fn unknown_methods_are_a_protocol_error() {
    let reply = sheetz::mcp::proto::handle_message(
        &json!({"jsonrpc":"2.0","id":3,"method":"nope"}).to_string(),
    )
    .expect("a reply");
    assert_eq!(reply["error"]["code"], -32601);
}

/// Notifications (no id) must not produce a response line.
#[test]
fn notifications_are_not_answered() {
    let reply = sheetz::mcp::proto::handle_message(
        &json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string(),
    );
    assert!(reply.is_none());
}

#[test]
fn garbage_input_does_not_panic() {
    assert!(sheetz::mcp::proto::handle_message("not json at all").is_none());
    assert!(sheetz::mcp::proto::handle_message("").is_none());
}

/// A tool call with no app behind it fails as an `isError` *result*, not a
/// protocol error — the model is meant to read it and adapt.
#[test]
fn tool_failures_come_back_as_results() {
    let reply = sheetz::mcp::proto::handle_message(
        &json!({"jsonrpc":"2.0","id":9,"method":"tools/call",
                "params":{"name":"workbook_info","arguments":{}}})
        .to_string(),
    )
    .expect("a reply");
    assert!(reply.get("error").is_none(), "must not be a protocol error");
    assert_eq!(reply["result"]["isError"], Value::Bool(true));
}

#[test]
fn the_socket_path_is_absolute_and_user_specific() {
    let path = sheetz::mcp::proto::socket_path();
    assert!(path.is_absolute());
    assert!(path.to_string_lossy().contains("sheetz"));
}
