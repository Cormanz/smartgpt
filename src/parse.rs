use std::{fmt::Display, collections::HashMap};

use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LLMResponse {
    #[serde(rename = "important takeaways: what was learned from the previous command, SPECIFIC and DETAILED")] 
    pub summary: Vec<Takeaway>,
    #[serde(rename = "goal information")]
    pub goal_information: GoalInformation,
    #[serde(rename = "command query")]
    pub command_query: String,
    #[serde(rename = "will I be completely done with the plan after this one query (true) or do I have more work to do (false)")]
    pub will_be_done_with_plan: bool
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Takeaway {
    pub takeaway: String,
    pub points: Vec<String>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub objective: String,
    #[serde(rename = "commands")] pub tasks: Vec<String>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoalInformation {
    #[serde(rename = "endgoal")] pub current_endgoal: String,
    #[serde(rename = "planned tasks")]
    pub plan: Vec<String>,
    #[serde(rename = "current step in plan")]
    pub step: usize
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandRequest {
    pub name: String,
    pub args: HashMap<String, String>
}