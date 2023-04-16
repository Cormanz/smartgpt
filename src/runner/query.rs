use serde::{Serialize, Deserialize};

use crate::{ScriptValue, Expression, Statement};

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

pub fn parse_query(commands: Vec<QueryCommand>) -> Vec<Statement> {
    commands.iter()
        .map(|cmd| Statement::Expression(parse_command(cmd.clone())))
        .collect()
}