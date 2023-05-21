use std::{error::Error, collections::VecDeque};

use serde::{Serialize, Deserialize};

use crate::{CommandContext, AgentInfo, Message, auto::{try_parse_json, run::Action, try_parse_yaml}, Weights};

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
pub struct RevisedMethodicalPlan {
    pub thoughts: String,
    pub solution: String,
    #[serde(rename = "revised remaining steps")]
    pub steps: Vec<MethodicalStep>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FinalResponse {
    pub response: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Memories {
    pub actions: Vec<String>,
    pub observations: Vec<String>
}

pub fn run_method_agent(
    context: &mut CommandContext, 
    get_agent: &impl Fn(&mut CommandContext) -> &mut AgentInfo,
    task: &str
) -> Result<String, Box<dyn Error>> {
    let agent = get_agent(context);

    agent.llm.clear_history();
    
    agent.llm.prompt.push(Message::System(format!(r#"
Tools:
google_search {{ "query": "query" }} - Gives you a list of URLs from a query.
browse_urls {{ "urls": [ "url 1", "url 2" ] }} - Read the text content from a URL.
file_write {{ "name": "file name", "lines": [ "line 1", "line 2 ] }} - Write content to a file.
final_response {{ "response": "response to user" }}

You have been given these tools.
"#).trim().to_string()));

    let observations = agent.observations.get_memories_sync(
        &agent.llm, task, 100, Weights {
            recall: 1.,
            recency: 1.,
            relevance: 1.
        }, 30
    )?;

    let observations = if observations.len() == 0 {
        "None found.".to_string()
    } else {
        observations.iter()
            .map(|obs| format!("- {}", obs.content))
            .collect::<Vec<_>>()
            .join("\n")
    };

    agent.llm.message_history.push(Message::User(format!(r#"
Here is your new task:
{task}

Here is a list of your memories:
{observations}

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
"#).trim().to_string()));

    let plan = try_parse_yaml::<MethodicalPlan>(&agent.llm, 2, Some(600), None)?;
    agent.llm.message_history.push(Message::Assistant(plan.raw));
    let plan = plan.data;
    log_yaml(&plan)?;

    drop(agent);

    let mut step_deque: VecDeque<MethodicalStep> = plan.steps.into();
    let mut completed_steps: Vec<MethodicalStep> = vec![];

    loop {
        let step = match step_deque.pop_front() {
            Some(step) => step,
            None => { break; }
        };

        let ind = completed_steps.len();

        let agent = get_agent(context);
        
        println!();

        let step_text = serde_yaml::to_string(&step)?;
        println!("{}", step_text);

        agent.llm.message_history.push(Message::User(format!(r#"
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
```
"#).trim().to_string()));

        let thoughts = try_parse_yaml::<MethodicalThoughts>(&agent.llm, 2, Some(1000), Some(0.5))?;
        agent.llm.message_history.push(Message::Assistant(thoughts.raw));
        let thoughts = thoughts.data;

        log_yaml(&thoughts)?;

        drop(agent);

        if thoughts.action.tool == "final_response" {
            let agent = get_agent(context);
            let final_response: FinalResponse = thoughts.action.args.unwrap().parse()?;

            agent.llm.message_history.pop();
            agent.llm.message_history.pop();

            agent.llm.message_history.push(Message::User(format!(r#"
Please list all important actions you took out.
Please also list all observations of information you have collected.

Respond in this YML format:
```yml
actions:
- A
- B

observations:
- A
- B
```
            "#).trim().to_string()));

            let memories = try_parse_yaml::<Memories>(&agent.llm, 2, Some(1000), Some(0.5))?.data;
            log_yaml(&memories);

            for memory in memories.actions.iter().chain(memories.observations.iter()) {
                agent.observations.store_memory_sync(&agent.llm, memory)?;
            }

            return Ok(final_response.response);
        }

        let out = use_tool(context, &|context| &mut context.agents.fast, thoughts.action.clone());
            
        println!();
        match out {
            Ok(out) => {
                println!("{out}");

                let agent = get_agent(context);
                agent.llm.message_history.push(Message::User(out));

                completed_steps.push(step.clone());
            },
            Err(err) => {
                println!("{err}");

                let completed_steps = serde_yaml::to_string(&MethodicalPlan { steps: completed_steps.clone() })?;

                let resp = format!(r#"
Tool use {} failed:
{err}

All of your previous steps, however were successful. Do not plan those out.

Think about what went wrong.
Then, replan all of the steps other than the ones completed.

Respond in this YML format:
```yml
thoughts: what went wrong
solution: what I will fix in my plan
revised remaining steps:
- idea: idea
  action:
    tool: precise tool name
    purpose: purpose
```
"#, thoughts.action.tool).trim().to_string();

                let agent = get_agent(context);
                agent.llm.message_history.push(Message::User(resp));

                let plan = try_parse_yaml::<RevisedMethodicalPlan>(&agent.llm, 2, Some(600), None)?.data;
                log_yaml(&plan);

                step_deque = plan.steps.into();
            }
        }
    }
    
    panic!("No final response");
}