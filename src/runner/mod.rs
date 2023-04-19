mod parse;
mod scriptvalue;
mod convert;

use std::{error::Error, collections::HashMap, time::Duration, fs, sync::{Mutex, Arc}};

use colored::Colorize;
pub use parse::*;
pub use scriptvalue::*;
pub use convert::*;

use tokio::{time::sleep, runtime::Handle};

use mlua::{
    AnyUserData, ExternalResult, Lua, Result as LuaResult, 
    UserData, UserDataMethods, Value, FromLua, Error as LuaError, Variadic, ToLua
};

use crate::{load_config, ProgramInfo, Command, Context, CommandContext};

pub async fn run_stuff(name: String, command: Command, context: &mut CommandContext, args: Vec<ScriptValue>) -> Result<ScriptValue, Box<dyn Error>> {
    println!("BURGON!");

    let result = command.run.invoke(context, args.clone()).await?;

    let args: Vec<Expression> = args.iter().map(|el| el.clone().into()).collect();
    let expr = Expression::FunctionCall(name.clone(), args);

    let json = serde_yaml::to_string(&result)
        .map_err(|_| GPTRunError("Could not parse ScriptValue as YAML.".to_string()))?;

    println!("{}", format!("Command {:?} was successful and returned:\n{}", expr, json));

    Ok(result)
}

pub fn test_runner() -> Result<(), Box<dyn Error>> {
    let lua = Lua::new();

    let config = fs::read_to_string("config.yml")?;
    let mut program = load_config(&config)?;

    let ProgramInfo { 
        name, personality: role, task, plugins,
        mut context, disabled_commands } = program;

    //println!("{}:", "Command Query".blue());

    for plugin in &plugins {
        for command in &plugin.commands {
            let name = command.name.clone();
            let command = command.box_clone();
            let lua_context_mutex = context.clone();
            let f = lua.create_function(move |lua, args: Variadic<_>| -> LuaResult<Value> {
                let args: Vec<ScriptValue> = args.iter()
                    .map(|el: &Value| el.clone())
                    .map(|el| ScriptValue::from_lua(el, lua))
                    .flat_map(|el| {
                        if let Ok(el) = el {
                            vec![ el ]
                        } else {
                            vec![]
                        }
                    })
                    .collect();
                println!("let us VERTCON!");
                
                let name = command.name.clone();
                let mut context = lua_context_mutex.lock().unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(async {
                    run_stuff(name.clone(), command.box_clone(), &mut context, args).await
                }).unwrap();

                Ok(result.to_lua(lua)?)
            })?;
            lua.globals().set(name, f)?;
            
        }
    }

    let _ = lua.load(r#"
    local articles = google_search("Mitosis Articles").items

    for i = 1, 3 do
      local article_url = articles[i].link
      local article_content = browse_article(article_url)
      file_append("mitosis_articles.txt", article_content)
    end
    "#).exec()?;

/*
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
    }*/

    Ok(())
}