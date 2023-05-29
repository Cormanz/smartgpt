<h1 align = "center">SmartGPT</h1>
<div align = "center">
    <a href="LICENSE.md">
        <img alt="License" src="https://img.shields.io/github/license/Cormanz/smartgpt?style=flat-square" />
        <img alt = "Stars" src="https://img.shields.io/github/stars/Cormanz/smartgpt?style=flat-square">
        <img src = "https://img.shields.io/badge/use-experimental-informational?style=flat-square">
    </a>
</div>

<hr/>

SmartGPT is an experimental program meant to provide LLMs (particularly GPT-3.5 and GPT-4) with the ability to complete complex tasks without user input by breaking them down into smaller problems, and collecting information using the internet and other external sources.

If you're interested in keeping up with the progress of SmartGPT, want to contribute to development, or have issues to discuss, [join the SmartGPT Discord](https://discord.gg/5uezFE2XES).

https://github.com/Cormanz/smartgpt/assets/32941017/11d737b4-9c93-4f22-b84f-d9c9d1ee0f9c

## Why?

There are many existing solutions to allowing LLMs to perform more complex tasks, such as [Auto-GPT](https://github.com/Torantulino/Auto-GPT) and [BabyAGI](https://github.com/yoheinakajima/babyagi). So, why SmartGPT?

- **Modularity**: With first class plugin support and the ability to compose Autos for whatever your project requires, SmartGPT is incredibly modular.

- **Flexibility**: SmartGPT has one `config.yml` file that is automatically generated where you can configure everything and anything.

- **Consistency**: SmartGPT has a smart system of dynamically executing actions and static tool-chaining to provide incredible consistent results.

There are two main shortcomings, however.

- **Ecosystem**: Due to its popularity, [AutoGPT](https://github.com/Torantulino/Auto-GPT) is a very polished and refined tool. It has many more tools and integrations with memory systems. To go with this, the codebase has been through large scrutiny, so it is generally less buggy and more tested than SmartGPT.

- **Memory Management**: Due to the extreme youth of this project, there is only one simple but limited memory system. However, this will change with time.

## Supporting Development

Currently, testing with SmartGPT is primarily being done with GPT3.5, and occasionally with GPT4, due to the costs of more-expensive models. As this project matures, we're aiming to experiment both with **multiple agents at once** and using **GPT4** much more to unleash maximum capabilities out of LLMs. This is expensive though, and as the core maintainer of SmartGPT, I'm still a high school student, and funding a project like this is difficult for me. If you're interest in helping push the boundaries of LLMs, [consider joining our Patreon.](https://www.patreon.com/SmartGPT)

## Disclaimer

SmartGPT is an **incredibly experimental** application. The goal is to unlock maximum potential out of LLMs, and stability is sacrificed for this. Backwards compatibility is a fever dream here. However, SmartGPT is also housing some of the most innovative ideas and experiments in the AutoGPT space right now, and although most are unsuccessful, a few hit the dart-board and stick.

## Quickstart

1. Install [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html), preferably the latest stable version.

2. Clone the repository with `git clone https://github.com/Cormanz/smartgpt.git && cd smartgpt`.

3. Run it in release mode with `cargo run --release`. This will create a `config.yml` for you.

4. Adjust the config to your liking, and execute it once again.

If you want more information, or would like to use SmartGPT as a crate in your own projects, [read the documentation](https://corman.gitbook.io/smartgpt/installation).

# How SmartGPT Works

## Autos

**Auto**s are the building blocks of SmartGPT. There are two types of Autos.

- **Runner**: A runner is given a single task, and is asked to complete it.
- **Assistants**: An Assistant Auto can be conversed with, and will give you responses back, in context of the conversation.

Assistants are highly experimental, so we recommend Runners.

An Auto will under the hood, run agent. An agent has two parts: The Dynamic Agent and The Static Agent.

## Dynamic Agent

The Dynamic Agent is the base agent. It runs a REACT-esque process, thinking, reasoning, and then making a decision. It can do one of three things:

- Brainstorm.
- Run an **action**.
- Give the user a final response.

When it runs an action, the Static Agent is dispatched to run the action.

## Static Agent

The Static Agent runs the subtasks given to it by the Dynamic Agent. Here's how it works:

1. It plans out each tool that is needed in the precise order to complete the task.
2. One by one, it'll run each step of the plan, filling in the arguments for the tool.

The Static Agent also saves assets that the Dynamic Agent can pass back to the Static Agent for future tasks.

## Memory

Agents all have **memory**. After completing a task, the agent will save a list of all observations into long-term memory. Once it starts another task, it will pull all long-term memories related to the task (using a VectorDB for this.)

## Plugin System

Autos can use a set of **tools** such as `google_search`, `browse_url`, etc. You define these using plugins. Plugins define their own set of tools, and can have their own data.

# License

`smartgpt` is available under the
[MIT license](https://opensource.org/licenses/MIT). See
[LICENSE](https://github.com/Cormanz/smartgpt/blob/main/LICENSE.md) for the full
license text.
