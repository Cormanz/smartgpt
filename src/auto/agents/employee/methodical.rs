use std::error::Error;

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_json, run::Action, try_parse_yaml}};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinalResponse {
    pub response: String
}

pub fn run_method_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str,
    first_inst: bool
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);
    
    if first_inst {
        agent.llm.prompt.push(Message::System(format!(r#"
Tools:
google_search {{ "query": "query" }} - Gives you a list of URLs from a query.
browse_urls {{ "urls": [ "url 1", "url 2" ] }} - Read the text content from a URL.
file_write {{ "name": "file name", "lines": [ "line 1", "line 2 ] }} - Write content to a file.
final_response {{ "response": "response to user" }}

You have been given these tools.
"#)));
    }

    agent.llm.message_history.push(Message::User(format!(r#"
Here is your new task:
{task}

Create a list of steps.
Start with an idea of what you need to do next and what tool could help.
Each step will use one tool.
Then, describe each time you will use the tool.
You will explain the exact purpose of using that specific tool.

Do not specify arguments.
Do not "repeat steps".

Respond in this YML format:
```yml
steps:
- idea: idea
  action:
    tool: precise tool name
    purpose: purpose
```
"#)));

    let plan = try_parse_yaml::<MethodicalPlan>(&agent.llm, 2, Some(300))?;
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

You must carry out this step with one entire action.
Include ALL information.

Respond in this YML format:
```yml
thoughts: thoughts
action:
    tool: tool
    args: {{}}
}}
```
"#)));

        let thoughts = try_parse_yaml::<MethodicalThoughts>(&agent.llm, 2, Some(1000))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data;

        log_yaml(&thoughts)?;

        drop(agent);

        if thoughts.action.tool == "final_response" {
            let final_response: FinalResponse = thoughts.action.args.unwrap().parse()?;
            return Ok(final_response.response);
        }

        let out = use_tool(context, &|context| &mut context.agents.fast, thoughts.action)?;
            
        println!();
        println!("{out}");

        let agent = get_agent(context);
        agent.llm.message_history.push(Message::User(out));
    }
    
    panic!("No final response");
}