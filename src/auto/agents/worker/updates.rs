use serde::{Deserialize, Serialize};

use super::{BrainThoughts, MethodicalPlan, MethodicalThoughts, MethodicalStep, Memories};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NamedAsset(pub String, pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DynamicUpdate {
    #[serde(rename = "plan")]
    Plan(String),
    #[serde(rename = "thoughts")]
    Thoughts(BrainThoughts)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StaticUpdate {
    #[serde(rename = "plan")]
    Plan(MethodicalPlan),
    #[serde(rename = "selected step")]
    SelectedStep(MethodicalStep),
    #[serde(rename = "thoughts")]
    Thoughts(MethodicalThoughts),
    #[serde(rename = "action results")]
    ActionResults(String),
    #[serde(rename = "selected asset")]
    SelectedAsset(String),
    #[serde(rename = "added asset")]
    AddedAsset(NamedAsset),
    #[serde(rename = "saving memories")]
    SavingMemories(),
    #[serde(rename = "added memories")]
    SavedMemories(Memories)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Update {
    #[serde(rename = "dynamic agent")]
    DynamicAgent(DynamicUpdate),
    #[serde(rename = "static agent")]
    StaticAgent(StaticUpdate)
}