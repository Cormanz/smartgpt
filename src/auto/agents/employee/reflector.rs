use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{ProgramInfo, Message, auto::{try_parse_json, agents::findings::{get_observations, get_reflections}}, LLM, Weights};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InitialObservations {
    #[serde(rename = "brainstormed loose plan")]
    pub idea: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reflections {
    #[serde(rename = "what have I done so far")]
    pub current_step: String,
    #[serde(rename = "having done this, what should my next step be to complete the task")]
    pub next_step: String,
    #[serde(rename = "quick plan for what to do next")]
    pub reflections: String,
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

Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "solve ..." }}
    Use pure mathematical equations.
browse_url {{ "url": "..." }}
    You can only read paragraph-only content from websites, you cannot interact with them.
file_append {{ "path": "...", "content": "..." }}
none {{}}

Remember that you use tools one at a time to complete your tasks.
You'll chain tools with arguments until you are successful.

Knowing that, brainstorm a very loose one sentence plan for how you will solve this.

Remember that you are working by yourself, and cannot work with anyone.

{{
    "brainstormed loose plan": "..."
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

First, consider what you have done so far in detail based on your observations.

Then, consider what you need to do in the future.

Finally, create a list of ideas for what to do next, to save to long-term memory.

Make sure you move onto a new step!

{{
    "what have I done so far": "..",
    "having done this, what should my next step be to complete the task": "..",
    "quick plan for what to do next": ".."
    "have I completed the task": true / false
}}

Respond in this JSON format.
"#
    )));

    try_parse_json(&context.agents.employee.llm, 2, Some(400)).map(|res| res.data)

}