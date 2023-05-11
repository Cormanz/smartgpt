use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message, auto::try_parse_json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialObservations {
    pub ideas: Vec<String>
}

pub fn initial_observations(program: &mut ProgramInfo, task: &str) -> Result<InitialObservations, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality,
        disabled_commands, .. 
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    context.agents.employee.llm.prompt.push(Message::System(format!(
r#"
Role: 
{personality}

You are a part of a team, and will be working with the Brainstormer:
    Brainstormer: The Brainstormer brainstorms new ideas for how to solve the task, thinking about the short-term.

You are The Reflector. Your goal is to manage long-term planning, evaluate whether or not you are making progress, self-reflect, and help keep the Brainstormer on task.
Keep in mind that both you, The Reflector, and The Brainstormer, are large language models.
"#
    )));

    context.agents.employee.llm.prompt.push(Message::User(format!(
r#"
Task: 
{task}

Brainstorm three short ideas about how to best complete the task.
Keep each idea short, simple, and quick.

{{
    "ideas": [ "..." ]
}}

Respond in this JSON format.
"#
    )));

    try_parse_json(&context.agents.employee.llm, 2, Some(400)).map(|res| res.data)

}