use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{Message, AgentInfo, CommandContext, auto::{try_parse_json, agents::employee::{log_yaml, run_react_agent}}};

use super::explain_results;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Step {
    pub step: String,
    pub tools: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanInfo {
    pub steps: Vec<Step>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FirstInstructInfo {
    pub instruction: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThoughtInstructInfo {
    pub criticism: String,
    #[serde(rename = "what I can learn from my criticism")]
    pub learned: String,
    pub step: String,
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

Tools:
google_search {{ "query": "..." }}
wolfram {{ "query": "..." }}
browse_url {{ "url": "..." }}
    Only use browse_url on websites that you have found from other searches.
file_append {{ "path": "...", "content": "..." }}

Task:
{task}

Break this task down into a list of steps. In each step, list the tool(s) you will use.

Respond in this exact JSON format:

{{
    "steps": [
        {{
             "step": "step",
             "tools": [ "tools" ]
        }}
    ]
}}
"#)));

    let plan_info = try_parse_json::<PlanInfo>(&agent.llm, 2, Some(600))?;
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

        println!("Back to the planner!");

        agent.llm.message_history.push(Message::User(format!(r#"
Your results from your Agent are:
{results}

Now, criticize your current workflow and plan. Then, learn from it.
Then, deduce what step you're on.

Then, decide if you are done.
If you are not done, give another instruction.
If you are done, set your instruction to `null`.

Feel free to have your Agent refine its previous result!

Respond in this exact JSON format:

```json
{{
    {{
        "criticism": "Constructive self-criticism",
        "what I can learn from my criticism": "What my criticism taught me",
        "step": "Precise step name (Tools to use)",
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