use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CanAnswerInfo {
    pub thoughts: String,
    #[serde[rename = "enough information to answer request"]] pub can_answer: bool
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MissingInfo {
    #[serde(rename = "missing information")] pub information: Vec<String>
}