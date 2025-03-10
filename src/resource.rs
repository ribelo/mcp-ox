use bon::Builder;
/// Resources that servers provide to clients
use chrono::{DateTime, Utc};
use mime::Mime;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] url::ParseError),
    #[error("Invalid file path")]
    InvalidFilePath,
    #[error("Resource not found")]
    NotFound,
}

/// Represents a resource in the extension with metadata
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Builder)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// URI representing the resource location (e.g., "file:///path/to/file" or "str:///content")
    #[builder(field)]
    pub uri: String,
    /// MIME type of the resource content ("text" or "blob")
    #[builder(field = "text/plain".to_string())]
    pub mime_type: String,
    /// Name of the resource
    #[builder(field = "unnamed".to_string())]
    pub name: String,
    /// Optional description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<String>,
}

impl<S: resource_builder::State> ResourceBuilder<S> {
    pub fn uri(mut self, uri: Url) -> Self {
        self.uri = uri.to_string();
        self
    }
    pub fn mime_type(mut self, mime_type: Mime) -> Self {
        self.mime_type = mime_type.to_string();
        self
    }
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    pub fn name_from_uri(mut self, uri: Url) -> Self {
        let name = uri
            .path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("unnamed")
            .to_string();
        self.name = name;
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ResourceContent {
    TextResourceContents {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        text: String,
    },
    BlobResourceContent {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        blob: String,
    },
}

impl Resource {
    /// Returns the scheme of the URI
    pub fn scheme(&self) -> Result<String, ResourceError> {
        let url = Url::parse(&self.uri)?;
        Ok(url.scheme().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_new_resource_with_file_uri() -> Result<(), ResourceError> {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();

        let uri =
            Url::from_file_path(temp_file.path()).map_err(|_| ResourceError::InvalidFilePath)?;

        let resource = Resource::builder().uri(uri).name("test").build();
        assert!(resource.uri.starts_with("file:///"));
        assert_eq!(resource.mime_type, "text");
        assert_eq!(resource.scheme()?, "file");

        Ok(())
    }

    #[test]
    fn test_resource_with_str_uri() -> Result<(), ResourceError> {
        let test_content = "Hello-world!";
        let uri = format!("str:///{}", test_content);

        let resource = Resource::builder()
            .uri(Url::parse(&uri).unwrap())
            .name("test.txt")
            .build();

        assert_eq!(resource.uri, uri);
        assert_eq!(resource.name, "test.txt");
        assert_eq!(resource.mime_type, "text");
        assert_eq!(resource.scheme()?, "str");

        Ok(())
    }

    #[test]
    fn test_mime_type_validation() -> Result<(), ResourceError> {
        // Test valid mime types
        let resource = Resource::builder()
            .uri(Url::parse("file:///test.txt").unwrap())
            .mime_type(mime::TEXT_PLAIN)
            .build();
        assert_eq!(resource.mime_type, "text/plain");

        let resource = Resource::builder()
            .uri(Url::parse("file:///test.bin").unwrap())
            .mime_type(mime::APPLICATION_OCTET_STREAM)
            .build();
        assert_eq!(resource.mime_type, "application/octet-stream");

        // Test default mime type
        let resource = Resource::builder()
            .uri(Url::parse("file:///test.txt").unwrap())
            .build();
        assert_eq!(resource.mime_type, "text");

        Ok(())
    }

    #[test]
    fn test_with_description() -> Result<(), ResourceError> {
        let mut resource = Resource::builder()
            .uri(Url::parse("file:///test.txt").unwrap())
            .name("test.txt")
            .build();

        resource.description = Some("A test resource".to_string());

        assert_eq!(resource.description, Some("A test resource".to_string()));
        Ok(())
    }

    #[test]
    fn test_resource_builder_with_mime_type() -> Result<(), ResourceError> {
        let resource = Resource::builder()
            .uri(Url::parse("file:///test.txt").unwrap())
            .name("test.txt")
            .mime_type(mime::APPLICATION_OCTET_STREAM)
            .build();

        assert_eq!(resource.mime_type, "application/octet-stream");

        Ok(())
    }

    #[test]
    fn test_invalid_uri() {
        let url_result = Url::parse("not-a-uri");
        assert!(url_result.is_err());
    }
    #[test]
    fn test_resource_serialization() {
        let resource = Resource::builder()
            .uri(Url::parse("https://example.com/data.json").unwrap())
            .name("test-json")
            .mime_type(mime::APPLICATION_JSON)
            .description("Test JSON resource")
            .build();

        let serialized = serde_json::to_string(&resource).unwrap();
        assert!(serialized.contains("\"uri\":\"https://example.com/data.json\""));
        assert!(serialized.contains("\"mimeType\":\"application/json\""));
        assert!(serialized.contains("\"name\":\"test-json\""));
        assert!(serialized.contains("\"description\":\"Test JSON resource\""));
    }

    #[test]
    fn test_resource_deserialization() {
        let json = r#"
        {
            "uri": "https://example.com/data.txt",
            "mimeType": "text/plain",
            "name": "example-text",
            "description": "A plain text file"
        }
        "#;

        let resource: Resource = serde_json::from_str(json).unwrap();
        dbg!(&resource);

        assert_eq!(resource.uri, "https://example.com/data.txt");
        assert_eq!(resource.mime_type, "text/plain");
        assert_eq!(resource.name, "example-text");
        assert_eq!(resource.description, Some("A plain text file".to_string()));
    }

    #[test]
    fn test_resource_content_serialization_text() {
        let content = ResourceContent::TextResourceContents {
            uri: "str:///content".to_string(),
            mime_type: Some("text/plain".to_string()),
            text: "Hello world".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        dbg!(&serialized);
        assert!(serialized.contains("\"uri\":\"str:///content\""));
        assert!(serialized.contains("\"mimeType\":\"text/plain\""));
        assert!(serialized.contains("\"text\":\"Hello world\""));
    }

    #[test]
    fn test_resource_content_serialization_blob() {
        let content = ResourceContent::BlobResourceContent {
            uri: "blob:///data".to_string(),
            mime_type: Some("application/octet-stream".to_string()),
            blob: "base64encodedcontent".to_string(),
        };

        let serialized = serde_json::to_string(&content).unwrap();
        dbg!(&serialized);
        assert!(serialized.contains("\"uri\":\"blob:///data\""));
        assert!(serialized.contains("\"mimeType\":\"application/octet-stream\""));
        assert!(serialized.contains("\"blob\":\"base64encodedcontent\""));
    }

    #[test]
    fn test_resource_content_deserialization() {
        let text_json = r#"
        {
            "uri": "str:///content",
            "mimeType": "text/plain",
            "text": "Hello world"
        }
        "#;

        let blob_json = r#"
        {
            "uri": "blob:///data",
            "mimeType": "application/octet-stream",
            "blob": "base64encodedcontent"
        }
        "#;

        let text_content: ResourceContent = serde_json::from_str(text_json).unwrap();
        let blob_content: ResourceContent = serde_json::from_str(blob_json).unwrap();

        match text_content {
            ResourceContent::TextResourceContents {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "str:///content");
                assert_eq!(mime_type, Some("text/plain".to_string()));
                assert_eq!(text, "Hello world");
            }
            _ => panic!("Expected TextResourceContents"),
        }

        match blob_content {
            ResourceContent::BlobResourceContent {
                uri,
                mime_type,
                blob,
            } => {
                assert_eq!(uri, "blob:///data");
                assert_eq!(mime_type, Some("application/octet-stream".to_string()));
                assert_eq!(blob, "base64encodedcontent");
            }
            _ => panic!("Expected BlobResourceContent"),
        }
    }
}
