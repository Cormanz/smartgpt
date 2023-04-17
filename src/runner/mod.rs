mod parse;
mod run;
mod query;
mod convert;

use std::{error::Error, collections::HashMap, time::Duration, fs};

use colored::Colorize;
pub use parse::*;
pub use run::*;
pub use convert::*;
pub use query::*;

use serde_json::Value;
use tokio::time::sleep;

use crate::{load_config, ProgramInfo};

pub async fn test_runner() -> Result<(), Box<dyn Error>> {
    let config = fs::read_to_string("config.yml")?;
    let mut program = load_config(&config).await?;

    let ProgramInfo { 
        name, personality: role, task, plugins,
        mut context, disabled_commands } = program;

    println!("{}:", "Command Query".blue());

    let query: QueryCommand = serde_yaml::from_str(
r#"
name: google_search
args:
- !Data examples of epigenetic modifications linked to cancer development
"#
    )?;

    println!("{}", serde_yaml::to_string(&query)?);
    
    sleep(Duration::from_secs(3)).await;

    println!();
    println!("{}", "Running Query".yellow());
    println!();

    context.command_out.clear();

    let query = parse_query(vec![ query ]);
    run_body(&mut context, &plugins, query).await?;

    for item in &context.command_out {
        println!("{}", item);
    }

    Ok(())
}