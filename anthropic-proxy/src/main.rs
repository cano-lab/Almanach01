use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use futures_util::{Stream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

const OPENAI_API_URL: &str = "https://api.augureai.ca/v1/chat/completions";
const DEFAULT_PORT: u16 = 8080;

#[derive(Clone)]
struct AppState {
    client: Client,
    api_key: String,
}

/// Anthropic API request format
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(default)]
    system: Option<AnthropicSystem>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    top_k: Option<u32>,
    #[serde(default)]
    stream: bool,
    #[serde(default)]
    tools: Option<Vec<AnthropicTool>>,
    #[serde(default)]
    tool_choice: Option<serde_json::Value>,
    #[serde(default)]
    stop_sequences: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thinking: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
    // Extra fields we don't translate — pass through as-is
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum AnthropicSystem {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

impl AnthropicSystem {
    fn to_openai_system_content(&self) -> String {
        match self {
            AnthropicSystem::Text(text) => text.clone(),
            AnthropicSystem::Blocks(blocks) => {
                blocks.iter()
                    .filter_map(|b| match b {
                        AnthropicContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("")
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct AnthropicMessage {
    role: String,
    content: AnthropicMessageContent,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
enum AnthropicMessageContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

impl AnthropicMessageContent {
    fn to_openai_content(&self) -> OpenAIMessageContent {
        match self {
            AnthropicMessageContent::Text(text) => {
                OpenAIMessageContent::Text(text.clone())
            }
            AnthropicMessageContent::Blocks(blocks) => {
                // Check if all blocks are text; if so, we can send a simple string
                let all_text = blocks.iter().all(|b| matches!(b, AnthropicContentBlock::Text { .. }));
                if all_text {
                    let text = blocks.iter()
                        .filter_map(|b| match b {
                            AnthropicContentBlock::Text { text } => Some(text.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    OpenAIMessageContent::Text(text)
                } else {
                    // Convert image blocks to OpenAI format
                    let parts: Vec<OpenAIContentPart> = blocks.iter()
                        .filter_map(|b| b.to_openai_part())
                        .collect();
                    OpenAIMessageContent::Array(parts)
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text { text: String },
    Image { source: AnthropicImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: Option<serde_json::Value>, is_error: Option<bool> },
}

impl AnthropicContentBlock {
    fn to_openai_part(&self) -> Option<OpenAIContentPart> {
        match self {
            AnthropicContentBlock::Text { text } => {
                Some(OpenAIContentPart::Text { text: text.clone() })
            }
            AnthropicContentBlock::Image { source } => {
                // Anthropic uses base64 with media_type; OpenAI uses url
                // For base64 images, we can embed as data URL
                let url = format!("data:{};base64,{}", source.media_type, source.data);
                Some(OpenAIContentPart::ImageUrl {
                    image_url: OpenAIImageUrl { url, detail: None },
                })
            }
            AnthropicContentBlock::ToolUse { id: _, name: _, input: _ } => {
                // Tool use from assistant becomes a tool_calls entry in the message
                None // handled separately in message conversion
            }
            AnthropicContentBlock::ToolResult { tool_use_id: _, content: _, is_error: _ } => {
                // Tool result goes in a tool role message
                None // handled separately in message conversion
            }
        }
    }

    fn to_text_string(&self) -> Option<String> {
        match self {
            AnthropicContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicTool {
    name: String,
    #[serde(default)]
    description: Option<String>,
    input_schema: serde_json::Value,
}

// ────────────────────────────────────────
// OpenAI format types
// ────────────────────────────────────────

#[derive(Debug, Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OpenAIMessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum OpenAIMessageContent {
    Text(String),
    Array(Vec<OpenAIContentPart>),
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum OpenAIContentPart {
    Text { text: String },
    ImageUrl { image_url: OpenAIImageUrl },
}

#[derive(Debug, Serialize, Clone)]
struct OpenAIImageUrl {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize)]
struct OpenAIFunction {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIToolCallFunction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIToolCallFunction {
    name: String,
    arguments: String,
}

// ────────────────────────────────────────
// Anthropic response types
// ────────────────────────────────────────

#[derive(Debug, Serialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    model: String,
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Serialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Serialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
}

#[derive(Debug, Serialize)]
struct AnthropicMessageStart {
    #[serde(rename = "type")]
    event_type: String,
    message: AnthropicResponse,
}

#[derive(Debug, Serialize)]
struct AnthropicContentBlockStart {
    #[serde(rename = "type")]
    event_type: String,
    index: usize,
    content_block: AnthropicContentBlock,
}

#[derive(Debug, Serialize)]
struct AnthropicContentBlockDelta {
    #[serde(rename = "type")]
    event_type: String,
    index: usize,
    delta: AnthropicDelta,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    InputJsonDelta { partial_json: String },
}

#[derive(Debug, Serialize)]
struct AnthropicContentBlockStop {
    #[serde(rename = "type")]
    event_type: String,
    index: usize,
}

#[derive(Debug, Serialize)]
struct AnthropicMessageDelta {
    #[serde(rename = "type")]
    event_type: String,
    delta: AnthropicMessageDeltaData,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Serialize, Default)]
struct AnthropicMessageDeltaData {
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessageStop {
    #[serde(rename = "type")]
    event_type: String,
}

#[derive(Debug, Serialize)]
struct AnthropicPing {
    #[serde(rename = "type")]
    event_type: String,
}

// ────────────────────────────────────────
// OpenAI response types
// ────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OpenAIChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: Option<OpenAIResponseMessage>,
    delta: Option<OpenAIResponseMessage>,
    finish_reason: Option<String>,
    logprobs: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
struct OpenAIResponseMessage {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
    tool_call_id: Option<String>,
    name: Option<String>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

fn convert_anthropic_to_openai(req: AnthropicRequest) -> OpenAIChatRequest {
    let mut messages: Vec<OpenAIMessage> = Vec::new();

    // Convert system message
    if let Some(system) = req.system {
        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: Some(OpenAIMessageContent::Text(system.to_openai_system_content())),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });
    }

    // Convert messages
    for msg in req.messages {
        let role = match msg.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => &msg.role,
        };

        let content = msg.content.to_openai_content();
        // If it's a simple text and we have tool_calls from a previous assistant message,
        // we need to handle that. But Anthropic tool_use/tool_result blocks are inside content.
        let mut tool_calls: Option<Vec<OpenAIToolCall>> = None;

        match &msg.content {
            AnthropicMessageContent::Blocks(blocks) => {
                // Check for tool_use blocks in assistant messages
                if role == "assistant" {
                    let tc: Vec<OpenAIToolCall> = blocks.iter()
                        .filter_map(|b| match b {
                            AnthropicContentBlock::ToolUse { id, name, input } => {
                                Some(OpenAIToolCall {
                                    id: id.clone(),
                                    tool_type: "function".to_string(),
                                    function: OpenAIToolCallFunction {
                                        name: name.clone(),
                                        arguments: input.to_string(),
                                    },
                                })
                            }
                            _ => None,
                        })
                        .collect();
                    if !tc.is_empty() {
                        tool_calls = Some(tc);
                    }
                }
                // Check for tool_result blocks — these become "tool" role messages
                if role == "user" {
                    let tool_results: Vec<AnthropicContentBlock> = blocks.iter()
                        .filter(|b| matches!(b, AnthropicContentBlock::ToolResult { .. }))
                        .cloned()
                        .collect();
                    if !tool_results.is_empty() {
                        for tr in tool_results {
                            if let AnthropicContentBlock::ToolResult { tool_use_id, content, .. } = tr {
                                let text = match content {
                                    Some(serde_json::Value::String(s)) => s,
                                    Some(v) => v.to_string(),
                                    None => "".to_string(),
                                };
                                messages.push(OpenAIMessage {
                                    role: "tool".to_string(),
                                    content: Some(OpenAIMessageContent::Text(text)),
                                    tool_calls: None,
                                    tool_call_id: Some(tool_use_id),
                                    name: None,
                                });
                            }
                        }
                        continue; // Skip adding this message; we added tool messages
                    }
                }
            }
            _ => {}
        }

        // Strip out tool_use blocks from content (they go in tool_calls)
        let content = match content {
            OpenAIMessageContent::Array(parts) => {
                let text_parts: Vec<String> = parts.iter()
                    .filter_map(|p| match p {
                        OpenAIContentPart::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect();
                if text_parts.is_empty() {
                    None
                } else {
                    Some(OpenAIMessageContent::Text(text_parts.join("")))
                }
            }
            other => Some(other),
        };

        // If content is empty but we have tool_calls, set content to None
        let content = match content {
            Some(OpenAIMessageContent::Text(ref s)) if s.is_empty() && tool_calls.is_some() => None,
            c => c,
        };

        messages.push(OpenAIMessage {
            role: role.to_string(),
            content,
            tool_calls,
            tool_call_id: None,
            name: None,
        });
    }

    let tools = req.tools.map(|tools| {
        tools.into_iter().map(|t| OpenAITool {
            tool_type: "function".to_string(),
            function: OpenAIFunction {
                name: t.name,
                description: t.description,
                parameters: t.input_schema,
            },
        }).collect()
    });

    let stop = if let Some(seqs) = req.stop_sequences {
        if seqs.is_empty() { None } else { Some(seqs) }
    } else {
        None
    };

    OpenAIChatRequest {
        model: req.model,
        messages,
        max_tokens: Some(req.max_tokens),
        temperature: req.temperature,
        top_p: req.top_p,
        stop,
        stream: req.stream,
        tools,
        tool_choice: req.tool_choice,
        stream_options: if req.stream {
            Some(serde_json::json!({"include_usage": true}))
        } else {
            None
        },
    }
}

fn convert_openai_to_anthropic(openai: OpenAIChatResponse) -> AnthropicResponse {
    let choice = openai.choices.into_iter().next();
    let (content, stop_reason, _tool_calls) = if let Some(choice) = choice {
        let content = choice.message.map(|m| {
            let mut blocks: Vec<AnthropicContentBlock> = Vec::new();
            if let Some(text) = m.content {
                if !text.is_empty() {
                    blocks.push(AnthropicContentBlock::Text { text });
                }
            }
            blocks
        }).unwrap_or_default();
        let stop_reason = choice.finish_reason.map(|r| match r.as_str() {
            "stop" => "end_turn".to_string(),
            "length" => "max_tokens".to_string(),
            "tool_calls" => "tool_use".to_string(),
            other => other.to_string(),
        });
        (content, stop_reason, None::<Vec<OpenAIToolCall>>)
    } else {
        (Vec::new(), None, None)
    };

    let usage = openai.usage.map(|u| AnthropicUsage {
        input_tokens: u.prompt_tokens,
        output_tokens: u.completion_tokens,
    }).unwrap_or(AnthropicUsage { input_tokens: 0, output_tokens: 0 });

    AnthropicResponse {
        id: openai.id,
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        model: openai.model,
        content,
        stop_reason,
        stop_sequence: None,
        usage,
    }
}

// ────────────────────────────────────────
// Streaming conversion
// ────────────────────────────────────────

fn stream_openai_to_anthropic(
    openai_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
    model: String,
) -> impl Stream<Item = Result<Bytes, std::convert::Infallible>> + Send + 'static {
    let mut buffer = String::new();
    let mut index: usize = 0;
    let mut has_started_block = false;
    let mut sent_message_start = false;
    let mut accumulated_text = String::new();
    let mut input_tokens: u32 = 0;
    let mut output_tokens: u32 = 0;
    let mut finished = false;

    openai_stream.map(move |result| {
        match result {
            Ok(bytes) => {
                if finished {
                    return Ok(Bytes::new());
                }
                buffer.push_str(&String::from_utf8_lossy(&bytes));
                let mut output = String::new();

                // Process complete SSE lines
                while let Some(pos) = buffer.find("\n\n") {
                    let chunk = buffer.drain(..=pos + 1).collect::<String>();
                    for line in chunk.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];
                            if data == "[DONE]" {
                                // Send message_delta with stop_reason
                                if sent_message_start {
                                    let delta = AnthropicMessageDelta {
                                        event_type: "message_delta".to_string(),
                                        delta: AnthropicMessageDeltaData {
                                            stop_reason: Some("end_turn".to_string()),
                                            stop_sequence: None,
                                        },
                                        usage: if input_tokens > 0 || output_tokens > 0 {
                                            Some(AnthropicUsage { input_tokens, output_tokens })
                                        } else {
                                            None
                                        },
                                    };
                                    if let Ok(json) = serde_json::to_string(&delta) {
                                        output.push_str(&format!("event: message_delta\ndata: {}\n\n", json));
                                    }
                                }
                                let stop = AnthropicMessageStop {
                                    event_type: "message_stop".to_string(),
                                };
                                if let Ok(json) = serde_json::to_string(&stop) {
                                    output.push_str(&format!("event: message_stop\ndata: {}\n\n", json));
                                }
                                finished = true;
                                break;
                            }
                            match serde_json::from_str::<OpenAIStreamChunk>(data) {
                                Ok(chunk) => {
                                    // Send message_start on first chunk
                                    if !sent_message_start {
                                        sent_message_start = true;
                                        let start = AnthropicMessageStart {
                                            event_type: "message_start".to_string(),
                                            message: AnthropicResponse {
                                                id: chunk.id.clone(),
                                                response_type: "message".to_string(),
                                                role: "assistant".to_string(),
                                                model: model.clone(),
                                                content: Vec::new(),
                                                stop_reason: None,
                                                stop_sequence: None,
                                                usage: AnthropicUsage { input_tokens: 0, output_tokens: 0 },
                                            },
                                        };
                                        if let Ok(json) = serde_json::to_string(&start) {
                                            output.push_str(&format!("event: message_start\ndata: {}\n\n", json));
                                        }
                                    }

                                    for choice in &chunk.choices {
                                        if let Some(delta) = &choice.delta {
                                            // Handle content
                                            if let Some(text) = &delta.content {
                                                if !text.is_empty() {
                                                    if !has_started_block {
                                                        has_started_block = true;
                                                        let block_start = AnthropicContentBlockStart {
                                                            event_type: "content_block_start".to_string(),
                                                            index,
                                                            content_block: AnthropicContentBlock::Text { text: "".to_string() },
                                                        };
                                                        if let Ok(json) = serde_json::to_string(&block_start) {
                                                            output.push_str(&format!("event: content_block_start\ndata: {}\n\n", json));
                                                        }
                                                    }
                                                    accumulated_text.push_str(text);
                                                    let delta_event = AnthropicContentBlockDelta {
                                                        event_type: "content_block_delta".to_string(),
                                                        index,
                                                        delta: AnthropicDelta::TextDelta { text: text.clone() },
                                                    };
                                                    if let Ok(json) = serde_json::to_string(&delta_event) {
                                                        output.push_str(&format!("event: content_block_delta\ndata: {}\n\n", json));
                                                    }
                                                }
                                            }

                                            // Handle tool_calls
                                            if let Some(tool_calls) = &delta.tool_calls {
                                                for tc in tool_calls {
                                                    if !has_started_block {
                                                        has_started_block = true;
                                                        let block_start = AnthropicContentBlockStart {
                                                            event_type: "content_block_start".to_string(),
                                                            index,
                                                            content_block: AnthropicContentBlock::ToolUse {
                                                                id: tc.id.clone(),
                                                                name: tc.function.name.clone(),
                                                                input: serde_json::Value::Null,
                                                            },
                                                        };
                                                        if let Ok(json) = serde_json::to_string(&block_start) {
                                                            output.push_str(&format!("event: content_block_start\ndata: {}\n\n", json));
                                                        }
                                                    }
                                                    if !tc.function.arguments.is_empty() {
                                                        let delta_event = AnthropicContentBlockDelta {
                                                            event_type: "content_block_delta".to_string(),
                                                            index,
                                                            delta: AnthropicDelta::InputJsonDelta { partial_json: tc.function.arguments.clone() },
                                                        };
                                                        if let Ok(json) = serde_json::to_string(&delta_event) {
                                                            output.push_str(&format!("event: content_block_delta\ndata: {}\n\n", json));
                                                        }
                                                    }
                                                }
                                            }

                                            // Handle finish_reason
                                            if let Some(reason) = &choice.finish_reason {
                                                if reason == "stop" || reason == "length" || reason == "tool_calls" {
                                                    // Content block stop
                                                    if has_started_block {
                                                        let block_stop = AnthropicContentBlockStop {
                                                            event_type: "content_block_stop".to_string(),
                                                            index,
                                                        };
                                                        if let Ok(json) = serde_json::to_string(&block_stop) {
                                                            output.push_str(&format!("event: content_block_stop\ndata: {}\n\n", json));
                                                        }
                                                        has_started_block = false;
                                                        index += 1;
                                                    }

                                                    // Message delta
                                                    let stop_reason = match reason.as_str() {
                                                        "stop" => Some("end_turn".to_string()),
                                                        "length" => Some("max_tokens".to_string()),
                                                        "tool_calls" => Some("tool_use".to_string()),
                                                        other => Some(other.to_string()),
                                                    };
                                                    let delta = AnthropicMessageDelta {
                                                        event_type: "message_delta".to_string(),
                                                        delta: AnthropicMessageDeltaData {
                                                            stop_reason,
                                                            stop_sequence: None,
                                                        },
                                                        usage: if input_tokens > 0 || output_tokens > 0 {
                                                            Some(AnthropicUsage { input_tokens, output_tokens })
                                                        } else {
                                                            None
                                                        },
                                                    };
                                                    if let Ok(json) = serde_json::to_string(&delta) {
                                                        output.push_str(&format!("event: message_delta\ndata: {}\n\n", json));
                                                    }

                                                    let stop = AnthropicMessageStop {
                                                        event_type: "message_stop".to_string(),
                                                    };
                                                    if let Ok(json) = serde_json::to_string(&stop) {
                                                        output.push_str(&format!("event: message_stop\ndata: {}\n\n", json));
                                                    }
                                                    finished = true;
                                                }
                                            }
                                        }
                                    }

                                    // Check for usage in extra fields
                                    if let Some(usage_val) = chunk.extra.get("usage") {
                                        if let Ok(usage) = serde_json::from_value::<OpenAIUsage>(usage_val.clone()) {
                                            input_tokens = usage.prompt_tokens;
                                            output_tokens = usage.completion_tokens;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("Failed to parse OpenAI stream chunk: {}", e);
                                }
                            }
                        }
                    }
                    if finished {
                        break;
                    }
                }

                Ok(Bytes::from(output))
            }
            Err(e) => {
                error!("Stream error: {}", e);
                Ok(Bytes::new())
            }
        }
    })
}

// ────────────────────────────────────────
// Handlers
// ────────────────────────────────────────

async fn handle_messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<AnthropicRequest>,
) -> Response {
    debug!("Received Anthropic request: {:?}", req);

    let openai_req = convert_anthropic_to_openai(req);

    let api_key = if let Some(auth) = headers.get("x-api-key") {
        auth.to_str().unwrap_or(&state.api_key).to_string()
    } else {
        state.api_key.clone()
    };

    let openai_req_body = match serde_json::to_string(&openai_req) {
        Ok(body) => body,
        Err(e) => {
            error!("Failed to serialize OpenAI request: {}", e);
            return error_response("invalid_request_error", &format!("Serialization error: {}", e));
        }
    };

    debug!("Forwarding to OpenAI: {}", openai_req_body);

    let request_builder = state.client
        .post(OPENAI_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", if openai_req.stream { "text/event-stream" } else { "application/json" })
        .body(openai_req_body);

    let upstream_resp = match request_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Request to upstream failed: {}", e);
            return error_response("api_error", &format!("Upstream request failed: {}", e));
        }
    };

    let status = upstream_resp.status();

    if openai_req.stream {
        // Streaming response
        let stream = upstream_resp.bytes_stream();
        let anthropic_stream = stream_openai_to_anthropic(stream, openai_req.model);

        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/event-stream")
            .header("cache-control", "no-cache")
            .header("connection", "keep-alive")
            .body(Body::from_stream(anthropic_stream))
            .unwrap()
    } else {
        // Non-streaming response
        let body_bytes = match upstream_resp.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Failed to read upstream response: {}", e);
                return error_response("api_error", &format!("Failed to read upstream response: {}", e));
            }
        };

        if !status.is_success() {
            let text = String::from_utf8_lossy(&body_bytes);
            warn!("Upstream returned error {}: {}", status, text);
            return Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(Body::from(body_bytes))
                .unwrap();
        }

        let openai_resp: OpenAIChatResponse = match serde_json::from_slice(&body_bytes) {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to parse OpenAI response: {}. Body: {}", e, String::from_utf8_lossy(&body_bytes));
                return error_response("api_error", &format!("Failed to parse upstream response: {}", e));
            }
        };

        let anthropic_resp = convert_openai_to_anthropic(openai_resp);

        match serde_json::to_string(&anthropic_resp) {
            Ok(json) => Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(json))
                .unwrap(),
            Err(e) => {
                error!("Failed to serialize Anthropic response: {}", e);
                error_response("api_error", &format!("Serialization error: {}", e))
            }
        }
    }
}

fn error_response(error_type: &str, message: &str) -> Response {
    #[derive(Serialize)]
    struct AnthropicError {
        #[serde(rename = "type")]
        error_type: String,
        error: ErrorDetail,
    }

    #[derive(Serialize)]
    struct ErrorDetail {
        #[serde(rename = "type")]
        error_type: String,
        message: String,
    }

    let err = AnthropicError {
        error_type: "error".to_string(),
        error: ErrorDetail {
            error_type: error_type.to_string(),
            message: message.to_string(),
        },
    };

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&err).unwrap_or_default()))
        .unwrap()
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let state = Arc::new(AppState {
        client: Client::new(),
        api_key,
    });

    let app = Router::new()
        .route("/v1/messages", post(handle_messages))
        .route("/health", axum::routing::get(health_check))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind to port");

    info!("Anthropic proxy listening on http://0.0.0.0:{}", port);
    info!("Forward target: {}", OPENAI_API_URL);
    info!("Usage: ANTHROPIC_BASE_URL=http://localhost:{} ANTHROPIC_API_KEY=<your-key> claude ...", port);

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
