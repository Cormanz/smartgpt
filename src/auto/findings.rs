use std::error::Error;

use crate::{LLM, Message};

use serde::{Deserialize, Serialize};

use super::try_parse_json;
#[derive(Serialize, Deserialize, Clone)]
pub struct FindingsReport {
    pub findings: Vec<String>,
    pub changes: Vec<String>
}

pub fn create_findings_prompt() -> String {
    format!(
r#"First, create a list of concise points about your findings from the commands.

Then, create a list of long-lasting changes that were executed (i.e. writing to a file, posting a tweet.) Use quotes when discussing specific details.

Keep your findings list very brief.

Respond in this exact format:

```json
{{
    "findings": [
      "A",
      "B"
    ],
    "changes": [
      "A",
      "B"
    ]
}}
```

Ensure your response is fully valid JSON."#)
}

pub fn ask_for_findings(llm: &mut LLM) -> Result<FindingsReport, Box<dyn Error>> {
    llm.message_history.push(Message::User(create_findings_prompt()));

    Ok(try_parse_json::<FindingsReport>(llm, 2, Some(300))?.data)
}

pub fn to_points(points: &[String]) -> String {
    points.iter()                
        .map(|el| format!("- {el}"))
        .collect::<Vec<_>>()
        .join("\n")
}