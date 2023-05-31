pub const DEFAULT_CONFIG: &str = r#"
personality: A superintelligent AI.
task: Write an essay on the Rust programming language.
agents:
    dynamic:
        llm:
            chatgpt:
                api key: PUT YOUR KEY HERE
                model: gpt-3.5-turbo
                embedding model: text-embedding-ada-002
        memory:
            local: {}
    planner:
        llm:
            chatgpt:
                api key: PUT YOUR KEY HERE
                model: gpt-3.5-turbo
                embedding model: text-embedding-ada-002
        memory:
            local: {}
    static:
        llm:
            chatgpt:
                api key: PUT YOUR KEY HERE
                model: gpt-3.5-turbo
                embedding model: text-embedding-ada-002
        memory:
            local: {}
    fast:
        llm:
            chatgpt:
                api key: PUT YOUR KEY HERE
                model: gpt-3.5-turbo
                embedding model: text-embedding-ada-002
        memory:
            local: {}
plugins:
    browse: {}
    google:
        cse id: PUT YOUR CSE ID HERE
        api key: PUT YOUR KEY HERE
    wolfram:
        app id: PUT YOUR APP ID HERE
    newsapi:
        api key: PUT YOUR KEY HERE
    brainstorm: {}
    #file system: {}
disabled tools: []
"#;