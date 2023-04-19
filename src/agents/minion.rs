use std::{error::Error, sync::{Arc, Mutex}};

use mlua::{Value, Variadic, Lua, Result as LuaResult, FromLua, ToLua};

use crate::{ProgramInfo, generate_commands, Message, Agents, ScriptValue, GPTRunError, Expression, Command, CommandContext};

/*pub async fn run_stuff(
    out: &mut String,
    name: String, command: Command, 
    context: &mut CommandContext, args: Vec<ScriptValue>
) -> Result<ScriptValue, Box<dyn Error>> {
    println!("BURGON!");

    let result = command.run.invoke(context, args.clone()).await?;

    let args: Vec<Expression> = args.iter().map(|el| el.clone().into()).collect();
    let expr = Expression::FunctionCall(name.clone(), args);

    let json = serde_yaml::to_string(&result)
        .map_err(|_| GPTRunError("Could not parse ScriptValue as YAML.".to_string()))?;

    out.push_str(&format!("Command {:?} was successful and returned:\n{}", expr, json));

    Ok(result)
}


pub fn run_script(program: &mut ProgramInfo, code: &str) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, .. 
    } = program;

    let lua = Lua::new();

    let context_mutex = Arc::new(context);
    let out_mutex = Arc::new(Mutex::new(String::new()));

    for plugin in plugins {
        for command in &plugin.commands {
            let name = command.name.clone();
            let command = command.box_clone();
            let lua_context_mutex = context_mutex.clone();
            let lua_out_mutex = out_mutex.clone();
            let f = lua.create_function(|lua, args: Variadic<_>| -> LuaResult<Value> {
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
                
                let name = command.name.clone();
                let mut context = lua_context_mutex.lock().unwrap();
                let mut out= lua_out_mutex.lock().unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                let result = rt.block_on(async {
                    run_stuff(&mut out, name.clone(), command.box_clone(), &mut context, args).await
                }).unwrap();

                Ok(result.to_lua(lua)?)
            })?;
            lua.globals().set(name, f)?;
            
        }
    }

    let _ = lua.load(code).exec()?;

    let out = out_mutex.lock().unwrap();
    Ok(out.clone())
}*/

pub fn run_minion(
    program: &mut ProgramInfo, task: &str, new_prompt: bool
) -> Result<String, Box<dyn Error>> {
    let ProgramInfo { 
        context, plugins, personality, 
        disabled_commands, .. 
    } = program;

    let ProgramInfo { context, plugins, personality, .. } = program;
    let mut context = context.lock().unwrap();

    let cmds = generate_commands(plugins, disabled_commands);

    context.agents.minion.prompt.push(Message::System(format!(
r#"
Using these commands:
{}

Write a LUA script to complete this task:
{}

Follow that plan exactly. Keep your LUA script as simple as possible.
"#,
        cmds, task
    )));

    let mut text = String::new();
    
    panic!();
}