# SmartGPT

SmartGPT is an experimental program meant to provide LLMs (particularly GPT-3.5 and GPT-4) with the ability to complete complex tasks without user input by breaking them down into smaller problems, and collecting information using the internet and other external sources.

<iframe width="560" height="315" src="https://www.youtube.com/embed/c9G1Cj_SCq0" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen></iframe>

## Disclaimer

SmartGPT isn't a ready-for-use application, it's an experiment by me, mostly for my own pleasure. It can also use a significant amount of tokens, and may run requests you didn't authorize, so it's not recommended to leave it running on its own for long periods of time.

## Objectives

Although inspired significantly by [Auto-GPT](https://github.com/Torantulino/Auto-GPT) and [BabyAGI](https://github.com/yoheinakajima/babyagi), SmartGPT has some key differences.

1. **Modularity** - The key difference between SmartGPT and other tools is that SmartGPT is completely modular. Everything can be replaced. SmartGPT relies on a system of plugins, where plugins can register new commands (allowing the AI to apply new actions and gain new inputs), add additional context to the beginning of the prompt, manage removed responses (due to token limit), and even manage their own data. Everything in SmartGPT, except the LLMs themselves, are plugins, including the access to the web, the memory, and even the ability to shut down the program. LLMs are also modular, and can be swapped out at any time.

2. **Prompting** - Part of the focus on SmartGPT is creating a single prompt that can allow the AI to easily run and exhibit complex behaviors and solve programs. Our prompting and the way we encode responses is meant to accomplish a few things.
- The AI reports on exactly what it learns from every command, allowing that information to both help it contextualize its thoughts and serve as long-term memory.
- The AI divides a problem into **objectives** and **tasks**, then focusing on one task at a time.

This area still needs work, because it often makes the AI overcomplexify tasks that are much simpler, but it has shown very promising results.

3. **Easy Configuration Management** - It's incredibly easy to configure your plugins in SmartGPT, both on the user side and the developer side. Users can save a very readable `config.yml` file (an example is shown in the root of the project directory), and run their entire project. Developers can easily parse these configurations using [Serde](https://serde.rs/).

## Main Task Loop

The main task loop of SmartGPT is as follows:

- Focuses on the current **endgoal** (one of the final goals it was told to complete)
- Records any findings from the **previous command**.
- Generates a list of **objectives**, usually 2 or 3. Each objective then has a list of **tasks** (usually one command each) to carry out.
- Chooses one **objective**, and particularly, one **task** from that objective to focus on.
- Generates an **idea** of what to do.
- Runs a **command**.

## How To Use

Prerequisites: [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

1. Clone the Repository.
```
git clone https://github.com/Cormanz/smartgpt.git
```

2. Run the Repository.
```
cargo run --release

# or alternatively

cargo build --release
target/release/smartgpt
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

    async fn apply_removed_response(&self, context: &mut CommandContext, response: &LLMResponse, cmd_output: &str, previous_response: bool) -> Result<(), Box<dyn Error>>;

    async fn create_data(&self, value: Value) -> Option<Box<dyn PluginData>>;
}
```

`create_context` defines whether or not the function will put extra text at the beginning of the prompt, and if so, what. This is mainly used to remind the LLM of what files it has, and what memories its pulled.

`apply_removed_responses` will apply the function whenever a response is updated, providing the `response` of what the Assistant said, and the result of its commands, `cmd_output`. `previous_response` is a redundant argument that will soon be removed.

`create_data` defines the long-term data that the plugin stores. Because of how Rust works, it's very tricky to convert the `PluginData` trait back into any one of its members, like `MemoryData`. Instead, you call invocations on `PluginData`, and parse out a response. Here's an example:

```rust
    invoke::<bool>(chatgpt_info, "push", ChatGPTMessage {
        role: ChatGPTRole::User,
        content: query.to_string()
    }).await?;
```

We take in our plugin data of `chatgpt_info`, tell it to `push` a new message, and it will return a `bool`. It's not the prettiest syntax, but decoupling plugin data from the rest of SmartGPT was one of the goals of the product, so this compromise was necessary (unless there's a better way to do this in Rust.)

# Areas of Development

This project isn't done as there's many more areas of development worth implementing:

- **Prompt Refining**: Ideally, I'd like for SmartGPT to more efficiently complete its tasks and not waste time on unnecessary queries. I'd also like to avoid having it gain tunnel-vision or repeat the same commands.
- **Implement More Plugins**: I'd like to implement more plugins for common features to make AI queries much easier.
- **Safe Terminal Access**: If this is possible, it would be very useful for running tasks on your computer.

# License

`smartgpt` is available under the
[MIT license](https://opensource.org/licenses/MIT). See
[LICENSE](https://github.com/Cormanz/smartgpt/blob/main/LICENSE) for the full
license text.