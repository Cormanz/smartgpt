use std::{fmt::Display, collections::HashMap};

use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LLMResponse {
    #[serde(rename = "important takeaways: what was learned from the previous command, SPECIFIC and DETAILED")] 
    pub summary: Vec<Takeaway>,
    #[serde(rename = "goal information")]
    pub goal_information: GoalInformation,
    #[serde(rename = "idea to complete current task")]
    pub idea: Option<String>,
    pub command: Option<CommandRequest>
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
    #[serde(rename = "chosen task")] pub current_task: Option<String>,
    #[serde(rename = "complete tasks")] pub complete_tasks: Vec<String>,
    #[serde(rename = "ongoing tasks")] pub ongoing_tasks: Vec<String>,
    #[serde(rename = "are all tasks complete")] pub end_goal_complete: bool
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandRequest {
    pub name: String,
    pub args: HashMap<String, String>
}