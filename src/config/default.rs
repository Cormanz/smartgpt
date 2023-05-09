pub const DEFAULT_CONFIG: &str = r#"
personality: A superintelligent AI.
type: !assistant
agents:
    employee:
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
    managers: []
plugins:
    browse: {}
    google:
        cse id: PUT YOUR CSE ID HERE
        api key: PUT YOUR KEY HERE
    file system: {}
    wolfram:
        app id: PUT YOUR APP ID HERE
    chatgpt:
        api key: PUT YOUR KEY HERE
    newsapi:
        api key: PUT YOUR KEY HERE
    wikipedia: {}
    none: {}
disabled commands: []
"#;