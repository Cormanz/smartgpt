mod parse;
mod run;

use std::{error::Error, collections::HashMap};

pub use parse::*;
pub use run::*;

pub async fn test_runner() -> Result<(), Box<dyn Error>> {
    let code = r#"
files = [ "a.txt", "b.txt", "c.txt" ]
for path in files:
    content = file_read(path)
    file_append("final.txt", content)
"#;

    let program = parse_gptscript(code)?;
    let mut ctx = ScriptContext {
        variables: HashMap::new()
    };
    run_body(&mut ctx, program).await?;

    Ok(())
}