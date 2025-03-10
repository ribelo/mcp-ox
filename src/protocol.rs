/// Implementation of the Model Context Protocol (MCP) using type-safe Rust.
///
/// This module provides a type-safe implementation of MCP over JSON-RPC 2.0,
/// enabling communication between clients and servers for AI model interactions.
use thiserror::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Protocol version for MCP.
pub const PROTOCOL_VERSION: &str = "0.2.0";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub error: ErrorData,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
    Error(JsonRpcError),
    Nil, // used to respond to notifications
}

// Standard JSON-RPC error codes
pub const PARSE_ERROR: i32 = -32700;
pub const INVALID_REQUEST: i32 = -32600;
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

/// Error information for JSON-RPC error responses.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrorData {
    /// The error type that occurred.
    pub code: i32,

    /// A short description of the error. The message SHOULD be limited to a concise single sentence.
    pub message: String,

    /// Additional information about the error. The value of this member is defined by the
    /// sender (e.g. detailed error information, nested errors etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

/// Error types that can occur in the MCP protocol.
#[derive(Debug, Clone, Error)]
pub enum ProtocolError {
    #[error("Transport error: {0}")]
    TransportError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("Method not implemented: {0}")]
    MethodNotImplemented(String),
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<ProtocolError> for ErrorData {
    fn from(error: ProtocolError) -> Self {
        match error {
            ProtocolError::TransportError(msg) => ErrorData {
                code: INTERNAL_ERROR,
                message: msg,
                data: None,
            },
            ProtocolError::ParseError(msg) => ErrorData {
                code: PARSE_ERROR,
                message: msg,
                data: None,
            },
            ProtocolError::ProtocolError(msg) => ErrorData {
                code: INVALID_REQUEST,
                message: msg,
                data: None,
            },
            ProtocolError::MethodNotImplemented(msg) => ErrorData {
                code: METHOD_NOT_FOUND,
                message: msg,
                data: None,
            },
            ProtocolError::InvalidParams(msg) => ErrorData {
                code: INVALID_PARAMS,
                message: msg,
                data: None,
            },
            ProtocolError::InternalError(msg) => ErrorData {
                code: INTERNAL_ERROR,
                message: msg,
                data: None,
            },
        }
    }
}
