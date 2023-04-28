pub fn create_findings_prompt() -> String {
    format!(
r#"First, create a list of concise points about your findings from the commands.

Then, create a list of long-lasting changes that were executed (i.e. writing to a file, posting a tweet.) Use quotes when discussing specific details.

Keep your findings list very brief.

Respond in this exact format:

```yml
findings:
- A
- B

changes:
- A
- B
```

Ensure your response is fully valid YAML."#)
}

fn to_points(points: &[String]) -> String {
    points.iter()                
        .map(|el| format!("- {el}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn create_letter(findings: &[String], changes: &[String]) -> String {
    format!(
"Dear Boss,

I have completed the tasks you assigned to me. These are my findings:
{}

These are the changes I had to carry out:
{}

Sincerely, Your Employee.",
        to_points(findings),
        to_points(changes)
    )
}