use serde::{Serialize, Deserialize};

use crate::{ScriptValue, Expression, Statement};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SimpleQueryCommand {
    name: String,
    args: Vec<ScriptValue>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct QueryCommand {
    name: String,
    args: Vec<CommandArg>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum CommandArg {
    Command(QueryCommand),
    Data(ScriptValue)
}

pub fn parse_simple_command(command: SimpleQueryCommand) -> Expression {
    let args: Vec<Expression> = command.args.iter().map(|arg| arg.clone().into()).collect();

    Expression::FunctionCall(
        command.name.clone(),
        args
    )
}

pub fn parse_command(command: QueryCommand) -> Expression {
    let args: Vec<Expression> = command.args.iter().map(|arg| {
        match arg {
            CommandArg::Command(cmd) => {
                parse_command(cmd.clone())
            }
            CommandArg::Data(data) => {
                data.clone().into()
            }
        }
    }).collect();

    Expression::FunctionCall(
        command.name.clone(),
        args
    )
}

pub fn parse_simple_query(commands: Vec<SimpleQueryCommand>) -> Vec<Statement> {
    commands.iter()
        .map(|cmd| Statement::Expression(parse_simple_command(cmd.clone())))
        .collect()
}

pub fn parse_query(commands: Vec<QueryCommand>) -> Vec<Statement> {
    commands.iter()
        .map(|cmd| Statement::Expression(parse_command(cmd.clone())))
        .collect()
}