# SmartGPT

SmartGPT is an experimental program meant to provide LLMs (particularly GPT-3.5 and GPT-4) with the ability to complete complex tasks without user input by breaking them down into smaller problems, and collecting information using the internet and other external sources.

If you're interested in keeping up with the progress of SmartGPT or contacting me, you can contact me on the [Octagon discord](https://discord.gg/rMnTeZWTBb), a hub for discussion and news of large language models and adjacent technologies.

https://github.com/Cormanz/smartgpt/assets/32941017/53bdcf83-4b2e-4798-b3f2-1a233b43c0e1

## Why?

There are many existing solutions to allowing LLMs to perform more complex tasks, such as [Auto-GPT](https://github.com/Torantulino/Auto-GPT) and [BabyAGI](https://github.com/yoheinakajima/babyagi). So, why SmartGPT?

- **Modularity**: SmartGPT is completely modular. It's designed in such a way that everything is completely separate, and you can easily compose Autos to your own needs. Other tools like AutoGPT have plugin systems, but SmartGPT was built from the ground up for plugin support.

- **Flexibility**: SmartGPT has one `config.yml` file that is automatically generated (TODO) holding all of the information for your use-case. In that config, you can easily change anything: which LLM you're using, which commands you want to allow, even how your Auto is structured.

- **Planning and Reasoning**: SmartGPT is incredibly experimental, and we have been constantly trying out new prompts, systems, and etc. Currently, we have a system where you can define a setup of **managers** and an **employee** to allow for recursive task planning and reasoning, providing high quality performance.

- **Configuration**: SmartGPT is incredibly easy to configure simply by using a simple `config.yml` file both for users, and for developers (who can parse their configurations using [Serde](https://serde.rs/))

There are two main shortcomings, however.

- **Ecosystem**: [AutoGPT](https://github.com/Torantulino/Auto-GPT) is a much more polished and refined tool, with many more commands and integrations with memory systems, as well as being much more well-tested than SmartGPT.

- **Memory Management**: As of right now, there is a very simple yet limited memory system in SmartGPT. Our plan is to refine this by adding first-class support for VectorDBs, and add in self-reflections.

## Disclaimer

SmartGPT is an **incredibly experimental** application. Our goal is to unlock maximum potential out of LLMs, and stability is sacrificed for this. Backwards compatibility is a no-no word here. Because of this, we also can't guarantee that using SmartGPT will be as intuitive as it could.

However, SmartGPT is also housing some of the most innovative ideas and experiments in the AutoGPT space right now, and although most are unsuccessful, a few hit the dart-board and stick.

## Quickstart

Prerequisites: [Rust and Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

1. Clone the Repository.
```
git clone https://github.com/Cormanz/smartgpt.git
```

Alternatively, [create a GitHub Codespace](https://github.com/codespaces/new?hide_repo_select=true&ref=main&repo=626190057) and run it there.

2. Run it your first time.
```
cargo run --release
```

Then, it will auto-generate a `config.yml`.

3. Fill in and optionally modify your `config.yml`, then run it again.
```
cargo run --release
```

[Read more in the Installation section of the documentation.](https://corman.gitbook.io/smartgpt/installation)

And that's it. You're done.

# How SmartGPT Works

## Autos

**Auto**s are the building blocks of SmartGPT. There are two types of Autos.

- **Runner**: A runner is given a single task, and is asked to complete it.
- **Assistants**: An Assistant Auto can be conversed with, and will give you responses back, in context of the conversation.

Assistants are highly experimental, so we recommend Runners.

Autos have **agents**. An agent is an LLM that handles planning, reasoning, and task execution. The Auto starts with your **top manager**, and asks it to run the task. Then, that manager will delegate tasks all the way down to your **employee**, which will run the tasks.

[Read more in the Autos section of the documentation.](https://corman.gitbook.io/smartgpt/autos/autos)

## Managers

Managers are a type of agent that plan and reason. They'll be given a task, and plan out that task into subtasks. Then, one subtask at a time, they'll delegate it down to their employee (a lower-level manager, or the task-running employee.)

[Read more in the Managers section of the documentation.](https://corman.gitbook.io/smartgpt/autos/agent-trees#managers)

## Employee

Employees are the lowest agent in the hierarchy. They're given a task, and they execute it one command at a time. They're much like the core application of AutoGPT, but they have a much more compact thought-loop.

[Read more in the Employees section of the documentation.](https://corman.gitbook.io/smartgpt/autos/agent-trees#employees)

## Memory

Agents all have **memory**. After completing a task, the agent will save a list of all observations into long-term memory. Once it starts another task, it will pull all long-term memories related to the task (using a VectorDB for this.)

[Read more in the Memory section of the documentation.](https://corman.gitbook.io/smartgpt/systems/memory)

## Plugin System

Autos can use a set of **tools** such as `google_search`, `browse_url`, etc. You define these using plugins. Plugins define their own set of commands, and can have their own data.

[Read more in the Plugin System section of the documentation.](https://corman.gitbook.io/smartgpt/systems/plugins)

# License

`smartgpt` is available under the
[MIT license](https://opensource.org/licenses/MIT). See
[LICENSE](https://github.com/Cormanz/smartgpt/blob/main/LICENSE.md) for the full
license text.
