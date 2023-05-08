use std::error::Error;

use crate::{ProgramInfo, Message, auto::{try_parse_json, ParsedResponse, employee::run_employee}};
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

pub fn run_manager(program: &mut ProgramInfo, layer: usize, task: &str) -> Result<(), Box<dyn Error>> {
    let ProgramInfo { 
        context, personality, ..
    } = program;
    let mut context = context.lock().unwrap();

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

    let dashes = "--".white();
    let manager = "Manager".blue();

    drop(context);
    loop {
        let ProgramInfo { 
            context, personality, ..
        } = program;
        let mut context = context.lock().unwrap();

        let thoughts = try_parse_json::<ManagerThought>(&context.agents.managers[layer].llm, 2, Some(400))?;
        let ParsedResponse { data: thoughts, raw } = thoughts;

        println!("{dashes} {manager} {dashes}");
        println!();
        println!("{}", serde_yaml::to_string(&thoughts)?);
        println!();

        match thoughts.action {
            ManagerAction::Delegate { task } => {
                if context.agents.managers.len() > layer + 1 {
                    drop(context);
                    run_manager(program, layer + 1, &task)?;
                } else {
                    drop(context);
                    run_employee(program, &task)?;
                }
            },
            ManagerAction::Finish {} => {
                return Ok(());
            }
        }
    }
}