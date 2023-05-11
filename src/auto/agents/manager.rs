use std::error::Error;

use crate::{ProgramInfo, Message, auto::{try_parse_json, ParsedResponse, agents::{findings::{ask_for_findings, to_points}, employee::run_employee}}, LLM, AgentInfo, Weights};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use super::findings::get_observations;

#[derive(Serialize, Deserialize)]
pub enum ManagerAction {
    #[serde(rename = "delegate one of the tasks")] Delegate {
        task: String
    },
    #[serde(rename = "finish")] Finish {}
}

#[derive(Serialize, Deserialize)]
pub struct ManagerThought {
    thoughts: String,
    reasoning: String,
    criticism: String,
    #[serde(rename = "do I need to revise my plan")]
    revise: bool,
    plan: Vec<String>,
    action: ManagerAction
}

pub fn run_manager<T>(program: &mut ProgramInfo, layer: usize, task: &str, end: impl Fn(&mut AgentInfo) -> T) -> Result<T, Box<dyn Error>> {
    let ProgramInfo { 
        context, personality, ..
    } = program;
    let mut context = context.lock().unwrap();
    let personality = personality.clone();

    context.agents.managers[layer].llm.prompt.clear();
    context.agents.managers[layer].llm.message_history.clear();

    context.agents.managers[layer].llm.prompt.push(Message::System(format!(
r#"
Personality: {personality}

You will be given one task.
Your goal is to split that task up into smaller tasks.
Your goal is to delegate those subtasks, one at a time.
Do it as fast as possible.
"#
    )));

    let observations = get_observations(&mut context.agents.managers[layer], task)?
        .unwrap_or("None found.".to_string());

    context.agents.managers[layer].llm.prompt.push(Message::User(format!(
"Here are your long-term memories:

{observations}"
    )));

    context.agents.managers[layer].llm.prompt.push(Message::User(format!(
        r#"
Your task is {task}.
        "#
            )));

    context.agents.managers[layer].llm.prompt.push(Message::User(format!(r#"
Reply in this format:

```json
{{
    "thoughts": "...",
    "reasoning": "...",
    "criticism": "...",
    "do I need to revise my plan": true / false,
    "plan": [
        "step A", 
        "step B", 
        "step C"
    ],
    "action": {{
        "delegate one of the tasks": {{
            "task": "..."
        }}
    }} or {{
        "finish": {{}}
    }}
}}
```

Ensure your delegated task is explained in detail.

Reply in that exact JSON format exactly.
Make sure every field is filled in detail.
Keep every field in that exact order.
"#)));

    context.agents.managers[layer].llm.message_history.push(Message::User(format!(
        r#"Plan out your subtasks. Then, delegate one task."#
    )));

    drop(context);

    let dashes = "--".white();
    let manager = "Manager".blue();

    loop {
        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();

        let thoughts = try_parse_json::<ManagerThought>(&context.agents.managers[layer].llm, 2, Some(400))?;
        let ParsedResponse { data: thoughts, raw } = thoughts;

        println!("{dashes} {manager} {dashes}");
        println!();
        println!("{}", serde_yaml::to_string(&thoughts)?);
        println!();

        let next_manager = context.agents.managers.len() > layer + 1;

        drop(context);
        let results = match thoughts.action {
            ManagerAction::Delegate { task } => {
                let results = if next_manager {
                    run_manager(program, layer + 1, &task, ask_for_findings)?
                } else {
                    run_employee(program, &task, ask_for_findings)?
                }?;

                results
            },
            ManagerAction::Finish {} => {
                break;
            }
        };

        let ProgramInfo { 
            context, ..
        } = program;
        let mut context = context.lock().unwrap();

        let findings_str = to_points(&results.findings);
        let changes_str = to_points(&results.changes);
        
        context.agents.managers[layer].llm.message_history.push(Message::Assistant(raw));
        
        context.agents.managers[layer].llm.message_history.push(Message::User(format!(
r#"Your task was completed.

Findings:
{findings_str}

Changes carried out:
{changes_str}
"#
        )));

        context.agents.managers[layer].llm.message_history.push(Message::User(format!(
            r#"Potentially refine your plan. Then, delegate one task (or finish.)"#
        )));

        let remaining_tokens = context.agents.managers[layer].llm.get_tokens_remaining(
            &context.agents.managers[layer].llm.get_messages()
        )?;

        if remaining_tokens < 1450 {
            ask_for_findings(&mut context.agents.managers[layer])?;
            context.agents.managers[layer].llm.crop_to_tokens_remaining(2600)?;

            let observations = get_observations(&mut context.agents.managers[layer], task)?
                .unwrap_or("None found.".to_string());
            context.agents.managers[layer].llm.prompt[1].set_content(&format!(
"Here are your long-term memories:

{observations}"
            ));
        }
        
        drop(context);
    }
    
    let ProgramInfo { 
        context, ..
    } = program;
    let mut context = context.lock().unwrap();

    return Ok(end(&mut context.agents.managers[layer]));
}