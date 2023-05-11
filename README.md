<h1 align = "center">SmartGPT</h1>
<div align = "center">
    <a href="LICENSE.md">
        <img alt="License" src="https://img.shields.io/github/license/Cormanz/smartgpt?style=flat-square" />
        <img alt = "Stars" src="https://img.shields.io/github/stars/Cormanz/smartgpt?style=social">
        <img src = "https://img.shields.io/badge/use-experimental-informational">
    </a>
</div>

<hr/>

SmartGPT is an experimental program meant to provide LLMs (particularly GPT-3.5 and GPT-4) with the ability to complete complex tasks without user input by breaking them down into smaller problems, and collecting information using the internet and other external sources.

If you're interested in keeping up with the progress of SmartGPT or contacting me, you can contact me on the [Octagon discord](https://discord.gg/rMnTeZWTBb), a hub for discussion and news of large language models and adjacent technologies.

https://github.com/Cormanz/smartgpt/assets/32941017/53bdcf83-4b2e-4798-b3f2-1a233b43c0e1

## Why?

There are many existing solutions to allowing LLMs to perform more complex tasks, such as [Auto-GPT](https://github.com/Torantulino/Auto-GPT) and [BabyAGI](https://github.com/yoheinakajima/babyagi). So, why SmartGPT?

- **Modularity**: With first class plugin support and the ability to compose Autos for whatever your project requires, SmartGPT is incredibly modular.

- **Flexibility**: SmartGPT has one `config.yml` file that is automatically generated where you can configure everything and anything.

- **Planning and Reasoning**: SmartGPT has an advanced hierarchical system of managers and employees to recursively break down your tasks.

- **Configuration**: SmartGPT is incredibly easy to configure simply by using a simple `config.yml` file both for users, and for developers.

There are two main shortcomings, however.

- **Ecosystem**: Due to its popularity, [AutoGPT](https://github.com/Torantulino/Auto-GPT) is a very polished and refined tool. It has many more commands and integrations with memory systems. To go with this, the codebase has been through large scrutiny, so it is generally less buggy and more tested than SmartGPT.

- **Memory Management**: Due to the extreme youth of this project, there is only one simple but limited memory system. However, this will change with time.

## Disclaimer

SmartGPT is an **incredibly experimental** application. The goal is to unlock maximum potential out of LLMs, and stability is sacrificed for this. Backwards compatibility is a fever dream here. However, SmartGPT is also housing some of the most innovative ideas and experiments in the AutoGPT space right now, and although most are unsuccessful, a few hit the dart-board and stick.

## Quickstart

0. Install [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html), preferably the latest stable version.

1. Clone the repository wih `git clone https://github.com/Cormanz/smartgpt.git && cd smartgpt`.

2. Run it in release mode with `cargo run --release`. This will create a `config.yml` for you.

3. Adjust the config to your liking, and execute it once again.


If you want more information, [read the documentation](https://corman.gitbook.io/smartgpt/installation).

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

## Employee

Employees are the lowest agent in the hierarchy. They're given a task, and they execute it one command at a time. They're much like the core application of AutoGPT, but they have a much more compact thought-loop.

## Memory

Agents all have **memory**. After completing a task, the agent will save a list of all observations into long-term memory. Once it starts another task, it will pull all long-term memories related to the task (using a VectorDB for this.)

## Plugin System

Autos can use a set of **tools** such as `google_search`, `browse_url`, etc. You define these using plugins. Plugins define their own set of commands, and can have their own data.

# License

`smartgpt` is available under the
[MIT license](https://opensource.org/licenses/MIT). See
[LICENSE](https://github.com/Cormanz/smartgpt/blob/main/LICENSE.md) for the full
license text.
