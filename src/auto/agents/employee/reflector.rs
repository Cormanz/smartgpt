use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message, auto::{try_parse_json, agents::findings::{get_observations, get_reflections}}, LLM, Weights};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialObservations {
    pub ideas: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reflections {
    pub progress: String,
    #[serde(rename = "next step")]
    pub next_step: String,
    pub reflections: Option<Vec<String>>,
    #[serde(rename = "have I completed the task")]
    pub task_complete: bool
}

pub fn prompt_reflector(llm: &mut LLM, personality: &str) {
    llm.prompt.push(Message::System(format!(
r#"
Role: 
{personality}

Your goal is to manage long-term planning, evaluate whether or not you are making progress, self-reflect, and stay on task.
Remember that you are a large language model.
"#
    )));
}

pub fn initial_observations(program: &mut ProgramInfo, task: &str) -> Result<InitialObservations, Box<dyn Error>> {
    let ProgramInfo { 
        context, personality, ..
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    prompt_reflector(&mut context.agents.employee.llm, &personality);

    context.agents.employee.llm.message_history.push(Message::User(format!(
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

pub fn self_reflect(program: &mut ProgramInfo, task: &str) -> Result<Reflections, Box<dyn Error>> {
    let ProgramInfo { 
        context, personality, ..
    } = program;
    let mut context = context.lock().unwrap();

    context.agents.employee.llm.prompt.clear();
    context.agents.employee.llm.message_history.clear();

    prompt_reflector(&mut context.agents.employee.llm, &personality);

    let observations = get_observations(&mut context.agents.employee, task, 10, Weights {
        recall: 0.,
        recency: 1.,
        relevance: 0.
    })?
        .unwrap_or("None found.".to_string());

    let reflections = get_reflections(&mut context.agents.employee, task, 7, Default::default())?
        .unwrap_or("None found.".to_string());

    context.agents.employee.llm.message_history.push(Message::User(format!(
r#"
Task: 
{task}

Reflections:
These are long-term reflections you have saved.
{reflections}

Observations:
These are observations that you have saved.
{observations}

First, examine what progress has been made through the recent observations you've made.

Then, determine what needs to be further done in the long-term to complete the task: your next step.

Finally, rephrase your next step into two or three shorter long-term reflections.

Make sure you move onto a new step!

{{
    "progress": "...",
    "next step": "...",
    "reflections": [ "..." ],
    "have I completed the task": true / false
}}

Respond in this JSON format.
"#
    )));

    try_parse_json(&context.agents.employee.llm, 2, Some(400)).map(|res| res.data)

}