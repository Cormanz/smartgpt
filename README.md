# SmartGPT

SmartGPT is an experimental program meant to provide LLMs (particularly GPT-3.5 and GPT-4) with the ability to complete complex tasks without user input by breaking them down into smaller problems, and collecting information using the internet and other external sources.

[Demonstration Video](https://www.youtube.com/watch?v=3EpmZ0-6sR0)

## Why?

There are many existing solutions to allowing LLMs to perform more complex tasks, such as [Auto-GPT](https://github.com/Torantulino/Auto-GPT) and [BabyAGI](https://github.com/yoheinakajima/babyagi). So, why SmartGPT?

- **Modularity**: SmartGPT is designed in such a way that you can easily add, remove, or toggle any part of it. Commands are abstracted into plugins, and LLMs are abstracted into their own interfaces that they have to implement.

- **Reasoning**: As far as I know, SmartGPT excels in reasoning tasks by far compared to other solutions, because it divides your task into multiple agents (Manager, Boss, Employee, Minion), and gives each agent a different task involving reasoning. This compartmentalization allows for much more impressive feats of reasoning. It also allows for you to potentially save on plenty of token-costs as context is split up between many of the agents, and you can use smaller models with the experimental LLAMA support potentially.

- **Configuration**: SmartGPT is incredibly easy to configure simply by using a simple `config.yml` file both for users, and for developers (who can parse their configurations using [Serde](https://serde.rs/))

There are two main shortcomings, however.

- **Ecosystem**: [AutoGPT](https://github.com/Torantulino/Auto-GPT) is a much more polished and refined tool, with many more commands and integrations with memory systems, as well as being much more well-tested than SmartGPT.

- **Memory Management**: As of right now, there is no memory system in SmartGPT. We're currently working to create a memory management system that would be much more flexible and work with multiple agents. However, even then, we'd still lack the ecosystem of memory management systems with different databases like Pinecone. This is an area that needs work.

## Disclaimer

SmartGPT isn't a ready-for-use application, it's an experiment by me, mostly for my own pleasure. It can also use a significant amount of tokens, and may run requests you didn't authorize, so it's not recommended to leave it running on its own for long periods of time. You are also held liable to the constraints of any services used with SmartGPT, i.e. OpenAI's GPT3, Wolfram Alpha, etc, if toggled and used.

It should also be noted that SmartGPT is a **very experimental** application that prioritizes rapid development over stability. Our goal is to pioneer the prompts and features of this, throwing ideas into the pool and seeing what floats, without any sort of priority on polishing, at least for now.

## Agents

SmartGPT has the following agents:

- **Manager**: Splits the main task into a few high-level subtasks, passing those to The Boss one by one.
- **Boss**: Takes its task and creates a loose plan, then splitting it into subtasks one by one, giving each subtask to the Employee.
- **Employee**: Takes its task, writes psuedo-code, passes it to The Minion.
- **Minion**: Refines the psuedo-code into a LUA script, runs it.

## LUA integration

SmartGPT is integrated with [LUA](https://www.lua.org/) to allow for simple scripts to be run. This is a massive improvement over existing frameworks, because they have to run each command one by one. However, this could still be unstable and may need work.

## How To Use

Note: Installing only seems to work on Linux due to some of the crate dependencies. Consider using [Windows Subsystem for Linux](https://learn.microsoft.com/en-us/windows/wsl/install) for Windows, or run SmartGPT in Github Codespaces.

Prerequisites: [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

1. Clone the Repository.
```
git clone https://github.com/Cormanz/smartgpt.git
```

2. Install Faiss _(if you don't use local long-term memory, you can skip this)_

Install FAISS as explained [here](https://github.com/Enet4/faiss-rs#installing-as-a-dependency)

If you still use the `memory` plugin without installing FAISS, it simply won't use the memory features. You'll know this because it won't log `Found Memories`.

3. Run the Repository.
```
cargo run --release
cargo run --release --features faiss
```

And that's it. You're done.

# Plugin System

The key benefit of SmartGPT is its plugin system, so I'll go depth into it here. A `Plugin` is defined as follows:

```rust
pub struct Plugin {
    pub name: String,
    pub cycle: Box<dyn PluginCycle>,
    pub dependencies: Vec<String>,
    pub commands: Vec<Command>
}
```

Plugins have a `name`, a set of `dependencies` for which plugins they require you also have, and a set of `commands` they register.

A `Command` is defined as follows:

```rust
pub struct Command {
    pub name: String,
    pub purpose: String,
    pub args: Vec<(String, String)>,
    pub run: Box<dyn CommandImpl>
}
```

Commands have a `name`, a `purpose`, and `args`. The latter two help describe how the function is used to the LLM. They also have a `run`, which is a dynamic trait that defines what happens when the command is used.

```rust
#[async_trait]
pub trait CommandImpl {
    async fn invoke(&self, ctx: &mut CommandContext, args: HashMap<String, String>) -> Result<String, Box<dyn Error>>;
}
```

`args` is provided as a `HashMap`. It's left as an exercise to the command-manager to parse those arguments, but usually, it's pretty easy using Rust's `?` operator.

Back to plugins, plugins also have a `cycle` dynamic trait, for a `PluginCycle`.

```rust
#[async_trait]
pub trait PluginCycle {
    async fn create_context(&self, context: &mut CommandContext, previous_prompt: Option<&str>) -> Result<Option<String>, Box<dyn Error>>;

    async fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>>;
}
```

`create_context` defines whether or not the function will put extra text at the beginning of the prompt, and if so, what. This is mainly used to remind the LLM of what files it has, and what memories its pulled.

`create_data` defines the long-term data that the plugin stores. Because of how Rust works, it's very tricky to convert the `PluginData` trait back into any one of its members, like `MemoryData`. Instead, you call invocations on `PluginData`, and parse out a response. Here's an example:

```rust
    invoke::<bool>(chatgpt_info, "push", ChatGPTMessage {
        role: ChatGPTRole::User,
        content: query.to_string()
    }).await?;
```

We take in our plugin data of `chatgpt_info`, tell it to `push` a new message, and it will return a `bool`. It's not the prettiest syntax, but decoupling plugin data from the rest of SmartGPT was one of the goals of the product, so this compromise was necessary (unless there's a better way to do this in Rust.)

# License

`smartgpt` is available under the
[MIT license](https://opensource.org/licenses/MIT). See
[LICENSE](https://github.com/Cormanz/smartgpt/blob/main/LICENSE) for the full
license text.
