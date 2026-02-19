#![allow(dead_code)]
//! Parameter and response types for Google Vids MCP tools.
//!
//! All param structs derive `Deserialize + JsonSchema` — never use raw `serde_json::Value`.
//! Tier: T2-C (μ Mapping + ∂ Boundary + ∃ Existence)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MCP Tool Param Structs
// ---------------------------------------------------------------------------

/// Param for tools that only need a presentation/video ID.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PresentationIdParam {
    /// The Google Vids/Slides presentation ID (from the URL).
    /// For Google Vids: the ID from `docs.google.com/videos/d/{ID}/edit`
    pub presentation_id: String,
}

/// Get elements on a specific scene/page.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetSceneParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// The page/scene object ID (from list_scenes).
    pub page_id: String,
}

/// Set text on a specific element within a scene.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetTextParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// The shape/text box object ID to modify.
    pub object_id: String,
    /// The new text content to set (replaces all existing text).
    pub text: String,
}

/// Find and replace text across all scenes.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReplaceTextParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// Text to search for (case-sensitive).
    pub find: String,
    /// Replacement text.
    pub replace_with: String,
    /// If true, match case exactly. Defaults to true.
    #[serde(default = "default_true")]
    pub match_case: bool,
}

/// Insert text at a specific position in a shape.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct InsertTextParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// The shape/text box object ID.
    pub object_id: String,
    /// The text to insert.
    pub text: String,
    /// Character index to insert at (0 = beginning). Omit to append.
    pub insertion_index: Option<u32>,
}

/// Add a new blank scene/slide.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AddSceneParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// Index where the new scene should be inserted (0-based).
    /// Omit to append at the end.
    pub insertion_index: Option<u32>,
    /// Optional layout to use for the new scene.
    /// Common values: "BLANK", "TITLE", "TITLE_AND_BODY", "CAPTION_ONLY"
    pub layout: Option<String>,
}

/// Delete a scene/slide.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteObjectParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// The page/element object ID to delete.
    pub object_id: String,
}

/// Create a text box on a scene.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateTextBoxParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// The page/scene to add the text box to.
    pub page_id: String,
    /// Text to insert into the new text box.
    pub text: String,
    /// X position in EMU (English Metric Units). 1 inch = 914400 EMU.
    #[serde(default = "default_x")]
    pub x_emu: i64,
    /// Y position in EMU.
    #[serde(default = "default_y")]
    pub y_emu: i64,
    /// Width in EMU.
    #[serde(default = "default_width")]
    pub width_emu: i64,
    /// Height in EMU.
    #[serde(default = "default_height")]
    pub height_emu: i64,
}

/// Raw batchUpdate request for advanced operations.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BatchUpdateParam {
    /// The Google Vids presentation ID.
    pub presentation_id: String,
    /// Array of request objects (Google Slides API batchUpdate format).
    /// Each object should be a single request like {"insertText": {...}}.
    pub requests: Vec<serde_json::Value>,
}

// Defaults
fn default_true() -> bool {
    true
}
fn default_x() -> i64 {
    457200 // 0.5 inch
}
fn default_y() -> i64 {
    457200 // 0.5 inch
}
fn default_width() -> i64 {
    7315200 // 8 inches
}
fn default_height() -> i64 {
    914400 // 1 inch
}

// ---------------------------------------------------------------------------
// Google Slides API Response Types
// ---------------------------------------------------------------------------

/// Top-level presentation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    #[serde(rename = "presentationId")]
    pub presentation_id: String,
    pub title: Option<String>,
    #[serde(default)]
    pub slides: Vec<Page>,
    #[serde(default)]
    pub masters: Vec<Page>,
    #[serde(default)]
    pub layouts: Vec<Page>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<PageSize>,
    pub locale: Option<String>,
}

/// A page (slide/scene) in the presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    #[serde(rename = "objectId")]
    pub object_id: String,
    #[serde(rename = "pageType")]
    pub page_type: Option<String>,
    #[serde(rename = "pageElements", default)]
    pub page_elements: Vec<PageElement>,
    #[serde(rename = "slideProperties")]
    pub slide_properties: Option<SlideProperties>,
    #[serde(rename = "pageProperties")]
    pub page_properties: Option<serde_json::Value>,
}

/// Properties of a slide (scene).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideProperties {
    #[serde(rename = "layoutObjectId")]
    pub layout_object_id: Option<String>,
    #[serde(rename = "masterObjectId")]
    pub master_object_id: Option<String>,
    #[serde(rename = "notesPage")]
    pub notes_page: Option<serde_json::Value>,
}

/// An element on a page (shape, image, video, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageElement {
    #[serde(rename = "objectId")]
    pub object_id: String,
    pub size: Option<Size>,
    pub transform: Option<serde_json::Value>,
    pub shape: Option<Shape>,
    pub image: Option<ImageElement>,
    pub video: Option<serde_json::Value>,
    #[serde(rename = "elementGroup")]
    pub element_group: Option<serde_json::Value>,
    pub table: Option<serde_json::Value>,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// A shape element (contains text).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    #[serde(rename = "shapeType")]
    pub shape_type: Option<String>,
    pub text: Option<TextContent>,
    pub placeholder: Option<Placeholder>,
}

/// Text content of a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "textElements", default)]
    pub text_elements: Vec<TextElement>,
}

/// A segment of text within a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextElement {
    #[serde(rename = "startIndex")]
    pub start_index: Option<u32>,
    #[serde(rename = "endIndex")]
    pub end_index: Option<u32>,
    #[serde(rename = "textRun")]
    pub text_run: Option<TextRun>,
    #[serde(rename = "paragraphMarker")]
    pub paragraph_marker: Option<serde_json::Value>,
    #[serde(rename = "autoText")]
    pub auto_text: Option<serde_json::Value>,
}

/// A run of text with uniform styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRun {
    pub content: Option<String>,
    pub style: Option<serde_json::Value>,
}

/// Placeholder information for a shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Placeholder {
    pub r#type: Option<String>,
    pub index: Option<u32>,
    #[serde(rename = "parentObjectId")]
    pub parent_object_id: Option<String>,
}

/// An image element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageElement {
    #[serde(rename = "contentUrl")]
    pub content_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    pub source_url: Option<String>,
    #[serde(rename = "imageProperties")]
    pub image_properties: Option<serde_json::Value>,
}

/// Size in EMU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

/// A dimension value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub magnitude: Option<f64>,
    pub unit: Option<String>,
}

/// Page size of the presentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageSize {
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
}

/// Response from batchUpdate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpdateResponse {
    #[serde(rename = "presentationId")]
    pub presentation_id: String,
    #[serde(default)]
    pub replies: Vec<serde_json::Value>,
    #[serde(rename = "writeControl")]
    pub write_control: Option<serde_json::Value>,
}

/// Service account JSON key format from GCP console.
#[derive(Debug, Clone, Deserialize)]
pub struct ServiceAccountKey {
    pub r#type: Option<String>,
    pub project_id: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: String,
    pub client_email: String,
    pub client_id: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
}

// ---------------------------------------------------------------------------
// Helper: extract all text from a page element
// ---------------------------------------------------------------------------

impl PageElement {
    /// Extract plain text from this element (if it contains text).
    pub fn text_content(&self) -> Option<String> {
        let shape = self.shape.as_ref()?;
        let text = shape.text.as_ref()?;
        let mut content = String::new();
        for elem in &text.text_elements {
            if let Some(ref run) = elem.text_run {
                if let Some(ref c) = run.content {
                    content.push_str(c);
                }
            }
        }
        if content.is_empty() {
            None
        } else {
            Some(content)
        }
    }

    /// Get the placeholder type if this is a placeholder element.
    pub fn placeholder_type(&self) -> Option<&str> {
        self.shape.as_ref()?.placeholder.as_ref()?.r#type.as_deref()
    }

    /// Get the shape type.
    pub fn shape_type(&self) -> Option<&str> {
        self.shape.as_ref()?.shape_type.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_presentation_id_param() {
        let json = r#"{"presentation_id":"abc123"}"#;
        let param: PresentationIdParam =
            serde_json::from_str(json).expect("deserialize PresentationIdParam");
        assert_eq!(param.presentation_id, "abc123");
    }

    #[test]
    fn deserialize_set_text_param() {
        let json = r#"{"presentation_id":"abc","object_id":"shape1","text":"Hello World"}"#;
        let param: SetTextParam = serde_json::from_str(json).expect("deserialize SetTextParam");
        assert_eq!(param.text, "Hello World");
    }

    #[test]
    fn deserialize_replace_text_param_defaults() {
        let json = r#"{"presentation_id":"abc","find":"old","replace_with":"new"}"#;
        let param: ReplaceTextParam =
            serde_json::from_str(json).expect("deserialize ReplaceTextParam");
        assert!(param.match_case); // default true
    }

    #[test]
    fn deserialize_add_scene_param_minimal() {
        let json = r#"{"presentation_id":"abc"}"#;
        let param: AddSceneParam = serde_json::from_str(json).expect("deserialize AddSceneParam");
        assert!(param.insertion_index.is_none());
        assert!(param.layout.is_none());
    }

    #[test]
    fn deserialize_create_text_box_defaults() {
        let json = r#"{"presentation_id":"abc","page_id":"p1","text":"test"}"#;
        let param: CreateTextBoxParam =
            serde_json::from_str(json).expect("deserialize CreateTextBoxParam");
        assert_eq!(param.x_emu, 457200);
        assert_eq!(param.width_emu, 7315200);
    }

    #[test]
    fn page_element_text_extraction() {
        let elem = PageElement {
            object_id: "shape1".into(),
            size: None,
            transform: None,
            shape: Some(Shape {
                shape_type: Some("TEXT_BOX".into()),
                text: Some(TextContent {
                    text_elements: vec![TextElement {
                        start_index: Some(0),
                        end_index: Some(5),
                        text_run: Some(TextRun {
                            content: Some("Hello".into()),
                            style: None,
                        }),
                        paragraph_marker: None,
                        auto_text: None,
                    }],
                }),
                placeholder: None,
            }),
            image: None,
            video: None,
            element_group: None,
            table: None,
            title: None,
            description: None,
        };
        assert_eq!(elem.text_content(), Some("Hello".to_string()));
    }

    #[test]
    fn page_element_no_text() {
        let elem = PageElement {
            object_id: "img1".into(),
            size: None,
            transform: None,
            shape: None,
            image: Some(ImageElement {
                content_url: Some("https://example.com/img.png".into()),
                source_url: None,
                image_properties: None,
            }),
            video: None,
            element_group: None,
            table: None,
            title: None,
            description: None,
        };
        assert!(elem.text_content().is_none());
    }

    #[test]
    fn batch_update_response_deserialize() {
        let json = r#"{"presentationId":"abc","replies":[{},{}]}"#;
        let resp: BatchUpdateResponse =
            serde_json::from_str(json).expect("deserialize BatchUpdateResponse");
        assert_eq!(resp.presentation_id, "abc");
        assert_eq!(resp.replies.len(), 2);
    }
}
