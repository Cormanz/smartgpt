use super::{BrainThoughts, MethodicalPlan, MethodicalThoughts, MethodicalStep, Memories};

#[derive(Clone, Debug)]
pub struct NamedAsset(pub String, pub String);

#[derive(Clone, Debug)]
pub enum DynamicUpdate {
    Plan(String),
    Thoughts(BrainThoughts)
}

#[derive(Clone, Debug)]
pub enum StaticUpdate {
    Plan(MethodicalPlan),
    SelectedStep(MethodicalStep),
    Thoughts(MethodicalThoughts),
    ActionResults(String),
    AddedAsset(NamedAsset),
    SavedMemories(Memories)
}

#[derive(Clone, Debug)]
pub enum Update {
    DynamicAgent(DynamicUpdate),
    StaticAgent(StaticUpdate)
}