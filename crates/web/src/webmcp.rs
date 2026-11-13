//! Bind the public use-webmcp-tool@0.3.0 API, not a replacement browser registry.
//! Chrome's recommended React integration owns detection and AbortSignal cleanup.
use js::{Error, Unknown};
use webapi::AbortSignal;

pub struct ToolOptions<F> {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Schema,
    pub annotations: Annotations,
    pub execute: F,
}

pub struct Schema {
    pub r#type: &'static str,
    pub properties: Properties,
    pub required: Vec<&'static str>,
    pub additional_properties: bool,
}

#[derive(Default)]
pub struct Properties {
    pub view: Option<ViewProperty>,
    pub moves: Option<TextProperty>,
    pub expected_revision: Option<IntegerProperty>,
    pub length: Option<IntegerProperty>,
}

pub struct ViewProperty {
    pub r#type: &'static str,
    pub r#enum: Vec<&'static str>,
}

pub struct Annotations {
    pub read_only_hint: bool,
}

pub struct ExecutionOptions {
    pub signal: &'static AbortSignal,
}

pub struct ToolState {
    pub supported: bool,
    pub registered: bool,
    #[cfg_attr(rust_js, rust_js::nullable)]
    pub error: Option<&'static Error>,
}

pub struct ToolResponse {
    pub content: Vec<TextContent>,
    pub is_error: bool,
}

pub struct TextContent {
    pub r#type: &'static str,
    pub text: String,
}

impl ToolResponse {
    pub fn text(text: impl Into<String>, is_error: bool) -> Self {
        Self {
            content: vec![TextContent {
                r#type: "text",
                text: text.into(),
            }],
            is_error,
        }
    }
}

#[cfg_attr(rust_js, rust_js::link_name = "use-webmcp-tool#useWebMCP")]
pub fn use_web_mcp<F>(options: ToolOptions<F>) -> ToolState
where
    F: Fn(Option<&Unknown>, ExecutionOptions) -> ToolResponse + 'static,
{
    unreachable!()
}

pub struct TextProperty {
    pub r#type: &'static str,
    pub description: &'static str,
    pub max_length: u32,
}
pub struct IntegerProperty {
    pub r#type: &'static str,
    pub minimum: u32,
    pub maximum: u32,
}
