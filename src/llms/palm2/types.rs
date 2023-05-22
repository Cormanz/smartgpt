use serde::{Serialize, Deserialize};

use crate::Message;



#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CitationSource {
    pub start_index: i32,
    pub end_index: i32,
    pub uri: String,
    pub license: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CitationMetadata {
    pub citation_sources: Vec<CitationSource>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PALMMessage {
    pub author: Option<String>,
    pub content: String,
    pub citation_metadata: Option<CitationMetadata>,
}

impl From<Message> for PALMMessage {
    fn from(message: Message) -> Self {
        let content = match message {
            Message::User(content) | Message::Assistant(content) | Message::System(content) => content,
        };

        PALMMessage {
            author: None,
            content,
            citation_metadata: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Example {
    pub input: PALMMessage,
    pub output: PALMMessage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessagePrompt {
    pub context: String,
    pub examples: Vec<Example>,
    pub messages: Vec<PALMMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateMessageResponse {
    pub prompt: MessagePrompt,
    pub temperature: f64,
    pub candidate_count: i32,
    pub top_p: f64,
    pub top_k: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CountTokensRequest {
    pub prompt: MessagePrompt,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextCompletion {
    pub output: String,
    pub safety_ratings: Vec<SafetyRating>,
    pub citation_metadata: Option<CitationMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlockedReason {
    BlockedReasonUnspecified,
    Safety,
    Other,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContentFilter {
    pub reason: BlockedReason,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory {
    HarmCategoryUnspecified,
    HarmCategoryDerogatory,
    HarmCategoryToxicity,
    HarmCategoryViolence,
    HarmCategorySexual,
    HarmCategoryMedical,
    HarmCategoryDangerous,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmBlockThreshold {
    HarmBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
    BlockNone,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmProbability {
    HarmProbabilityUnspecified,
    Negligible,
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafetySetting {
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafetyRating {
    pub category: HarmCategory,
    pub probability: HarmProbability,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SafetyFeedback {
    pub rating: SafetyRating,
    pub setting: SafetySetting,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTextResponse {
    pub candidates: Option<Vec<TextCompletion>>,
    pub filters: Option<Vec<ContentFilter>>,
    pub safety_feedback: Option<Vec<SafetyFeedback>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmbedTextRequest {
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GCPModel {
    pub name: String,
    pub base_model_id: Option<String>,
    pub version: String,
    pub display_name: String,
    pub description: String,
    pub input_token_limit: i32,
    pub output_token_limit: i32,
    pub supported_generation_methods: Vec<String>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenCountResponse {
    pub token_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextPrompt {
    // Add the fields of TextPrompt object here
    pub text: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTextRequest {
    pub prompt: TextPrompt,
    pub safety_settings: Vec<SafetySetting>,
    pub stop_sequences: Vec<String>,
    pub temperature: f64,
    pub candidate_count: i32,
    pub max_output_tokens: i32,
    pub top_p: f64,
    pub top_k: i32,
}

#[derive(Deserialize, Debug)]
pub struct Embedding {
    pub value: Vec<f32>,
}

#[derive(Deserialize, Debug)]
pub struct EmbeddingResponse {
    pub embedding: Option<Embedding>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModelResponse {
    pub models: Vec<GCPModel>,
    pub next_page_token: Option<String>,
}