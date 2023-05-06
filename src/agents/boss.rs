use std::{error::Error, fmt::{Display, Debug}};
use crate::{ProgramInfo, AgentLLMs, Agents, Message, agents::{process_response, LINE_WRAP, run_employee, Choice, try_parse_yaml, try_parse_json}, Weights, AgentInfo};
use colored::Colorize;
use serde::{Serialize, Deserialize, __private::de};

use super::EmployeeResponse;

#[derive(Clone)]
pub enum Task {
    Task(String),
    Feedback(String, String)
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::Task(task) => {
                write!(f, "Your task is: {task:?}")
            },
            Task::Feedback(task, feedback) => {
                write!(f, "Your initial task was: {task:?}\nYou must refine your task results with this feedback: {feedback:?}")
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct BossDecisionInfo {
    #[serde(rename = "new employee request")] pub new_request: Option<String>,
    #[serde(rename = "report to manager")] pub report: Option<String>
}


#[derive(Serialize, Deserialize)]
pub struct BossDecision {
    #[serde(rename = "action info")] pub info: BossDecisionInfo,
    #[serde(rename = "new loose plan")] pub loose_plan: Option<String>,
    #[serde(rename = "memory query")] pub memory_query: String,
    pub observations: Option<Vec<String>>
}


#[derive(Debug, Clone)]
pub struct NoManagerRequestError;

impl Display for NoManagerRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "could not parse.")
    }
}

impl Error for NoManagerRequestError {}

pub fn run_boss_once(
    program: &mut ProgramInfo, task: Task, 
    previous_loose_plan: Option<String>,
    previous_request: Option<String>,
    previous_employee_response: Option<EmployeeResponse>,
    memory_query: Option<String>
) -> Result<BossDecision, Box<dyn Error>> {
    let ProgramInfo { context, plugins, personality, .. } = program;
    let mut context = context.lock().unwrap();

    context.agents.boss.llm.prompt.clear();
    context.agents.boss.llm.message_history.clear();

    let commands = plugins.iter()
        .flat_map(|el| el.commands.iter())
        .map(|el| el.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    context.agents.boss.llm.prompt.push(Message::System(format!(
"You are The Boss, a large language model.

Personality: {}

You have been assigned one task by The Manager, a large language model. You will use your loose planning and adaptability to complete this task.
Your goal is to quickly and efficiently get the task done without refining it too much. You just want to get a sort of quicker, shallower answer.
Complete your task as quickly as possible.

You have access to one employee named The Employee, a large language model, who will help you run tasks.
Your Employee has access to the following resources: {}

Your Employee is not meant to do detailed work, but simply to help you find information.

Only ask The Employee for one thing at a time.
Keep your Employee requests very simple.
Make sure to tell the Employee to save important information to files.

You must only act through your employee.
You cannot do any work on your own.
Your only goal is to handle loose planning and going from one step to the next through your Employee.

Info on Employee Requests:
Do not give your employee specific commands, simply phrase your request with natural language.
Provide a very narrow and specific request for the Employee.
Remember: Your Employee is not meant to do detailed work, but simply to help you find information.
Make sure to tell the Employee to save important information to files!

Info on Memory Queries:
Your memory query is a very short summary of every topic in your mind that is relevant at this moment.
Think of it like a search query.
Your memory query will be used to help you find relevant observations and reflections.

Info on Loose Plans:
Loose plans are very unstructured, loose, high-level plans. 
They are rapidly updated whenever necessary.
They are only two short sentences."
        , personality, commands
    )));
    
    let AgentInfo { llm, observations, .. } = &mut context.agents.boss;
    let observations = observations.get_memories_sync(
        &llm,
        memory_query.as_deref().unwrap_or("None"),
        200,
        Weights {
            recall: 1.,
            recency: 1.,
            relevance: 1.
        },
        50
    )?;
    let observation_text = if observations.len() == 0 {
        "None found.".to_string()
    } else {
        observations.iter().enumerate()
            .map(|(ind, observation)| format!("{ind}. {}", observation.content))
            .collect::<Vec<_>>()
            .join("\n")
    };

    drop(llm);
    drop(observations);

    let formatted_observations = process_response(&observation_text, LINE_WRAP);

    println!("{}", "BOSS".blue());
    println!("{}", "The boss has found its observations.".white());
    println!();
    println!("{formatted_observations}");
    println!();

    let previous_loose_plan = previous_loose_plan.unwrap_or("None".to_string());
    let previous_request = previous_request.unwrap_or("None".to_string());

    context.agents.boss.llm.message_history.push(Message::User(format!(
"TASK
{task:?}

PREVIOUS LOOSE PLAN
{previous_loose_plan}

PREVIOUS REQUEST
    Request: {previous_request}
    Response from Empoloyee: {}

OBSERVATIONS
{observation_text}",
        previous_employee_response.map(|resp| resp.report).as_deref().unwrap_or("None")
    )));

    context.agents.boss.llm.message_history.push(Message::User(format!(
r#"Respond in this format:

```json
{{
    "observations": [
      "A",
      "B"
    ],
    "new loose plan": "I should...",
    "memory query": "I am working on...",
    "action info": {{
      "reasoning": "Reasoning",
      "decision": "report to manager" / "new employee request",
      "report to manager": "Dear Manager...",
      "new employee request": "Can you try..."
    }}
}}
```

All fields must be specified exactly as shown above.
Keep all of the requests in the EXACT SAME JSON FORMAT.
If you do not want to put a specific field, put the field, but set its value to `null`.

Ensure your response is in the exact JSON format as specified."#)));

    let (response, decision) = try_parse_json::<BossDecision>(&context.agents.boss.llm, 2, Some(1000))?;
    context.agents.boss.llm.message_history.push(Message::Assistant(response.clone()));

    let formatted_response = process_response(&response, LINE_WRAP);

    println!("{}", "BOSS".blue());
    println!("{}", "The boss has made a decision.".white());
    println!();
    println!("{formatted_response}");
    println!();

    Ok(decision)
}

pub fn run_boss(
    program: &mut ProgramInfo, task: Task
) -> Result<String, Box<dyn Error>> {
    let mut previous_loose_plan: Option<String> = None;
    let mut previous_request: Option<String> = None;
    let mut previous_employee_response: Option<EmployeeResponse> = None;
    let mut memory_query: Option<String> = None;
    let mut new_prompt = match task {
        Task::Feedback(_, _) => false,
        Task::Task(_) => true
    };
    loop {
        let decision = run_boss_once(
            program, task.clone(),
            previous_loose_plan.clone(),
            previous_request.clone(), previous_employee_response.clone(),
            memory_query.clone()
        )?;

        /*if let Some(observations) = decision.observations.clone() {
            for observation in observations {
                let mut context = program.context.lock().unwrap();
                let AgentInfo { llm, observations, .. } = &mut context.agents.boss;
                observations.store_memory_sync(llm, &observation)?;
            }
        }*/

        if let Some(report) = decision.info.report {
            return Ok(report);
        }

        if let Some(loose_plan) = decision.loose_plan {
            previous_loose_plan = Some(loose_plan);
        }

        if let Some(request) = decision.info.new_request {
            let employee_response = run_employee(
                program, &request, new_prompt, 
                previous_employee_response.map(|el| el.memory_query)
            )?;

            for observation in &employee_response.observations {
                let mut context = program.context.lock().unwrap();
                let AgentInfo { llm, observations, .. } = &mut context.agents.boss;
                observations.store_memory_sync(llm, &observation)?;
            }

            previous_request = Some(request);
            previous_employee_response = Some(employee_response);
        }

        memory_query = Some(decision.memory_query);
    }
}