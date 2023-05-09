use std::error::Error;

use crate::{ProgramInfo, Message, auto::{try_parse_json, ParsedResponse, agents::{findings::{ask_for_findings, to_points}, employee::run_employee}}, LLM, AgentInfo, Weights};
use colored::Colorize;
use serde::{Deserialize, Serialize};

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
    plan: String,
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

    let AgentInfo { observations, llm, .. } = &mut context.agents.managers[layer];
    let observations = observations.get_memories_sync(
        &llm,
        task,
        200,
        Weights {
            recall: 1.,
            recency: 1.,
            relevance: 1.
        },
        50
    )?;
    let observations = if observations.len() == 0 {
        None
    } else {
        Some(observations.iter().enumerate()
            .map(|(ind, observation)| format!("{ind}. {}", observation.content))
            .collect::<Vec<_>>()
            .join("\n"))
    };

    if let Some(observations) = observations {
        context.agents.managers[layer].llm.prompt.push(Message::User(format!(
"Here are your long-term memories:

{observations}"
        )))
    }

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
    "plan": "...",
    "action": {{
        "delegate one of the tasks": {{
            "task": "..."
        }}
    }} or {{
        "finish": {{}}
    }}
}}
```

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

        drop(context);
    }
    
    let ProgramInfo { 
        context, ..
    } = program;
    let mut context = context.lock().unwrap();

    return Ok(end(&mut context.agents.managers[layer]));
}