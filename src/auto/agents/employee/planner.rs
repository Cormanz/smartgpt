use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{Message, AgentInfo, CommandContext, auto::{try_parse_json, agents::employee::{log_yaml, run_react_agent}}};

use super::explain_results;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanInfo {
    #[serde(rename = "broad overarching stages")]
    pub stages: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FirstInstructInfo {
    pub instruction: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThoughtInstructInfo {
    pub reflection: String,
    pub stage: String,
    #[serde(rename = "explanation of why or why not my task is complete")]
    pub explanation: String,
    pub instruction: Option<String>
}

pub fn run_planner_agent(
    context: &mut CommandContext, 
    task: &str
) -> Result<String, Box<dyn Error>> {
    let agent = &mut context.agents.planner;
    agent.llm.clear_history();

    agent.llm.message_history.push(Message::System(format!("
You are a Planner.
You will work with an Agent.
The Agent is not smart, so you must talk very simply to it.
")));

    agent.llm.message_history.push(Message::User(format!(r#"
You have access to an Agent. Your agent can use external resources to help complete its task.
You will work with your Agent, and your Agent will use its resources to complete each task you give it.

Task:
{task}"

List broad, overarching stages to accomplish your task.
Stages should not be specific or complex.

There should be three or four stages.

Respond in this exact JSON format:

{{
    "broad overarching stages": [ "Stage" ]
}}
"#)));

    let plan_info = try_parse_json::<PlanInfo>(&agent.llm, 2, Some(300))?;
    log_yaml(&plan_info.data)?;

    agent.llm.message_history.push(Message::Assistant(plan_info.raw));
    agent.llm.message_history.push(Message::User(format!(r#"
Now, you instruct your Agent, keep your instruction simple, one sentence.
Do not tell the Agent about your overarching task.

Respond in this exact JSON format:

```json
{{
    {{
        "instruction": "Instruction"
    }}
}}
```
"#)));

    let instruction_info = try_parse_json::<FirstInstructInfo>(&agent.llm, 2, Some(300))?;
    log_yaml(&instruction_info.data)?;

    agent.llm.message_history.push(Message::Assistant(instruction_info.raw));

    drop(agent);
    context.agents.react.llm.clear_history();

    let mut results = run_react_agent(context, &|ctx| &mut ctx.agents.react, &instruction_info.data.instruction, true)?;

    loop {
        println!("{results}");
        println!();

        let agent = &mut context.agents.planner;

        agent.llm.message_history.push(Message::User(format!(r#"
Your results from your Agent are:
{results}

Now, self-reflect on what results you got, how it helps you with your task, and what to do next.
Then, deduce what stage of your four-staged plan you're on.

Then, decide if you are done.
If you are not done, give another instruction.
If you are done, set your instruction to `null`.

Feel free to have your Agent refine its previous result!

Respond in this exact JSON format:

```json
{{
    {{
        "reflection": "Constructive self-reflection",
        "stage": "Precise stage name",
        "explanation of why or why not my task is complete": "Explanation",
        "instruction": "Instruction"
    }}
}}
```
"#)));

        println!("Generating next instruction...");

        let instruction_info = try_parse_json::<ThoughtInstructInfo>(&agent.llm, 2, Some(300))?;
        log_yaml(&instruction_info.data)?;

        agent.llm.message_history.push(Message::Assistant(instruction_info.raw));

        match instruction_info.data.instruction {
            None => {
                break;
            }
            Some(instruction) => {
                results = run_react_agent(context, &|ctx| &mut ctx.agents.react, &instruction, false)?;
            }
        }
    }

    explain_results(context, &|ctx| &mut ctx.agents.planner)
}