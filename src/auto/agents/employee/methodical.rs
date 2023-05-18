use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_json, run::Action}};

use super::{log_yaml, use_tool};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalThoughts {
    pub thoughts: String,
    pub action: Action
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalAction {
    pub tool: String,
    pub purpose: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalStep {
    pub idea: String,
    pub action: MethodicalAction
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MethodicalPlan {
    pub steps: Vec<MethodicalStep>
}

pub fn run_method_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str,
    first_inst: bool
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    
    if first_inst {
        agent.llm.prompt.push(Message::User(format!(r#"
Tools:
google_search {{ "query": "..." }}
browse_urls {{ "urls": [ "..." ] }}
file_append {{ "path": "...", "content": "..." }}

You have been given these tools.

Task:
Research at least three articles on M&M health detriments.

Create a list of steps.
Each step will use one tool.
Then, describe each time you will use the tool (i.e. browsing multiple articles)
Do not specify arguments.

Respond in this JSON format:
```json
{{
    "steps": [
        {{
            "idea": "idea",
            "action": {{
                "tool": "tool",
                "purpose": "purpose"
            }}
        }}
    ]
}}
```
"#)));
    }

    let plan = try_parse_json::<MethodicalPlan>(&agent.llm, 2, Some(300))?;
    agent.llm.prompt.push(Message::Assistant(plan.raw));
    let plan = plan.data;
    log_yaml(&plan)?;

    drop(agent);

    for (ind, step) in plan.steps.iter().enumerate() {
        let agent = get_agent(context);
        
        println!();

        let step_text = serde_yaml::to_string(&step)?;
        println!("{}", step_text);

        agent.llm.message_history.push(Message::User(format!(r#"
You have created a plan.
Now you will carry out the next step: 
{step_text}

Respond in this JSON format:
```json
{{
    "thoughts: "thoughts",
    "action": {{
        "tool": "tool",
        "args": {{ ... }}
    }}
}}
```
"#)));

        let thoughts = try_parse_json::<MethodicalThoughts>(&agent.llm, 2, Some(600))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data;

        log_yaml(&thoughts)?;

        drop(agent);

        let out = use_tool(context, &|context| &mut context.agents.fast, thoughts.action)?;
            
        println!();
        println!("{out}");

        let agent = get_agent(context);
        agent.llm.message_history.push(Message::User(out));
    }

    Ok("No".to_string())
}