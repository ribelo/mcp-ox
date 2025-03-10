use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bon::Builder;
use mime::Mime;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

/// Error types for prompt operations
#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Other error: {0}")]
    Other(String),
}

/// A prompt or prompt template that the server offers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    /// A list of arguments to use for templating the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(field)]
    pub arguments: Option<Vec<Value>>,
    /// The name of the prompt or prompt template
    #[builder(into)]
    pub name: String,

    /// An optional description of what this prompt provides
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<String>,
}

impl<S: prompt_builder::State> PromptBuilder<S> {
    pub fn argument<T: JsonSchema>(mut self) -> PromptBuilder<S> {
        if let Some(args) = &mut self.arguments {
            args.push(schema_for!(T).to_value());
        } else {
            self.arguments = Some(vec![schema_for!(T).to_value()]);
        }
        self
    }
}

/// Represents the role of a message sender in a prompt conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptMessageRole {
    User,
    Assistant,
}

/// Text provided to or from an LLM
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    /// The text content of the message
    pub text: String,
}

/// An image provided to or from an LLM
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ImageContent {
    /// The base64-encoded image data
    #[builder(field)]
    pub data: String,

    /// The MIME type of the image
    #[builder(field)]
    pub mime_type: String,
}

impl<S: image_content_builder::State> ImageContentBuilder<S> {
    pub fn data(mut self, data: impl Into<String>) -> Result<Self, PromptError> {
        let data_str = data.into();

        // Validate base64 data
        BASE64_STANDARD.decode(&data_str).map_err(|_| {
            PromptError::InvalidParameters("Image data must be valid base64".to_string())
        })?;

        self.data = data_str;
        Ok(self)
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Result<Self, PromptError> {
        let mime_type_str = mime_type.into();

        // Validate mime type
        if !mime_type_str.starts_with("image/") {
            return Err(PromptError::InvalidParameters(
                "MIME type must be a valid image type (e.g. image/jpeg)".to_string(),
            ));
        }

        self.mime_type = mime_type_str;
        Ok(self)
    }
}

/// The contents of a specific resource or sub-resource
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContents {
    /// The URI of this resource
    #[builder(field)]
    pub uri: String,

    /// The MIME type of this resource, if known
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(field)]
    pub mime_type: Option<String>,
}

impl<S: resource_contents_builder::State> ResourceContentsBuilder<S> {
    pub fn uri(mut self, uri: Url) -> Self {
        // Url is already a valid URL type, no need to validate
        self.uri = uri.to_string();
        self
    }

    pub fn mime_type(mut self, mime_type: Mime) -> Self {
        let mime_type_str: String = mime_type.to_string();

        // No need to validate the mime_type parameter since it's already a valid Mime type
        self.mime_type = Some(mime_type_str);
        self
    }
}

/// Text resource contents with the actual text data
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Builder)]
#[serde(rename_all = "camelCase")]
pub struct TextResourceContents {
    /// The URI of this resource
    #[builder(field)]
    pub uri: String,

    /// The MIME type of this resource, if known
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(field)]
    pub mime_type: Option<String>,

    /// The text content of the resource
    #[builder(into)]
    pub text: String,
}

impl<S: text_resource_contents_builder::State> TextResourceContentsBuilder<S> {
    pub fn uri(mut self, uri: Url) -> Self {
        // Url is already a valid URL type, no need to validate
        self.uri = uri.to_string();
        self
    }

    pub fn mime_type(mut self, mime_type: Mime) -> Self {
        let mime_type_str: String = mime_type.to_string();

        // No need to validate the mime_type parameter since it's already a valid Mime type
        self.mime_type = Some(mime_type_str);
        self
    }
}


/// The contents of a resource, embedded into a prompt or tool call result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct EmbeddedResource {
    /// The resource content
    pub resource: TextResourceContents,
}

/// Content types that can be included in prompt messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PromptMessageContent {
    /// Plain text content
    Text(TextContent),

    /// Image content with base64-encoded data
    Image(ImageContent),

    /// Embedded server-side resource
    Resource { resource: EmbeddedResource },
}

impl Default for PromptMessageContent {
    fn default() -> Self {
        Self::Text(TextContent::default())
    }
}

/// Describes a message returned as part of a prompt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct PromptMessage {
    /// The content of the message
    #[builder(field)]
    pub content: PromptMessageContent,
    /// The role of the message sender
    pub role: PromptMessageRole,
}

impl<S: prompt_message_builder::State> PromptMessageBuilder<S> {
    pub fn content(
        mut self,
        content: impl Into<PromptMessageContent>,
    ) -> Result<Self, PromptError> {
        let content = content.into();

        // Validate base64 data
        if let PromptMessageContent::Image(image) = &content {
            BASE64_STANDARD.decode(&image.data).map_err(|_| {
                PromptError::InvalidParameters("Image data must be valid base64".to_string())
            })?;

            // Validate mime type
            if !image.mime_type.starts_with("image/") {
                return Err(PromptError::InvalidParameters(
                    "MIME type must be a valid image type (e.g. image/jpeg)".to_string(),
                ));
            }
        }

        self.content = content;
        Ok(self)
    }
}

impl PromptMessage {
    /// Create a new image message with the given role, data and mime type
    pub fn new_image<S: Into<String>>(
        role: PromptMessageRole,
        data: S,
        mime_type: S,
    ) -> Result<Self, PromptError> {
        let data = data.into();
        let mime_type = mime_type.into();

        // Validate base64 data
        BASE64_STANDARD.decode(&data).map_err(|_| {
            PromptError::InvalidParameters("Image data must be valid base64".to_string())
        })?;

        // Validate mime type
        if !mime_type.starts_with("image/") {
            return Err(PromptError::InvalidParameters(
                "MIME type must be a valid image type (e.g. image/jpeg)".to_string(),
            ));
        }

        Ok(Self {
            role,
            content: PromptMessageContent::Image(ImageContent { data, mime_type }),
        })
    }

    /// Create a new resource message with the given role, URI, mime type, and text
    pub fn new_resource(
        role: PromptMessageRole,
        uri: String,
        mime_type: Option<String>,
        text: String,
    ) -> Self {
        let resource_contents = TextResourceContents {
            uri,
            mime_type,
            text,
        };

        Self {
            role,
            content: PromptMessageContent::Resource {
                resource: EmbeddedResource {
                    resource: resource_contents,
                },
            },
        }
    }
}
