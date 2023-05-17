# Welcome to the SmartGPT contributing guide

Thank you for taking the time to contribute to SmartGPT! Here, we describe the process for submitting contributions and provide some useful tips for making your contribution as effective as possible.

If you do not follow , your pull request will not be considered.

## Getting started

1. Fork the project repository on GitHub and clone it locally (or just clone if you have write access).
2. Make sure that you are working off of the latest commit by `git pull`ing from upstream.
3. Create a branch in which to make your changes--name it descriptively so other contributors know what it is!
4. Work on your changes, commit them, and push them up to origin (your repository). Please include meaningful commit messages that help maintainers understand what your commit does.
5. When you're ready, submit a pull request against the original repository.

## Formatting
All code must be uniform in order to keep this project from growing archaic. We require the use of [`rustfmt`](https://github.com/rust-lang/rustfmt), Rust's built in compiler. 

All of rustfmt's features are not immediately usable, some of them are considered unstable and require a Rust nightly. So, after installing Rust nightly, execute `cargo +nightly fmt`.
