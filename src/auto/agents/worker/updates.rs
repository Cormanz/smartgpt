use super::{BrainThoughts, MethodicalPlan, MethodicalThoughts};

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
    Thoughts(MethodicalThoughts),
    ActionResults(String),
    AddedAssets(Vec<NamedAsset>)
}

#[derive(Clone, Debug)]
pub enum Update {
    DynamicAgent(DynamicUpdate),
    StaticAgent(StaticUpdate)
}