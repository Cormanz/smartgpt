use std::{fmt::Display, collections::HashMap};

use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LLMResponse {
    #[serde(rename = "important takeaways: what was learned from the previous command, SPECIFIC and DETAILED")] 
    pub summary: Vec<Takeaway>,
    #[serde(rename = "goal information")]
    pub goal_information: GoalInformation,
    #[serde(rename = "gptscript command query")]
    pub command_query: String
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
    #[serde(rename = "are all queries complete")] pub end_goal_complete: bool
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandRequest {
    pub name: String,
    pub args: HashMap<String, String>
}