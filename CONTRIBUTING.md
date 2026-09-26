# Contribution Guide

Thank you for considering contributing. Please review the guidelines below before making a contribution.

To ensure that your contributions are considered, please follow these guidelines. Contributions that do not adhere to these guidelines may not be accepted.

## Before You Start

Please review the [FAQ](https://lusingander.github.io/serie/faq/index.html), [Goals](https://lusingander.github.io/serie/introduction/index.html#goals), and [Non-Goals](https://lusingander.github.io/serie/introduction/index.html#non-goals) before starting work.

You may open a pull request directly for documentation corrections, small isolated fixes with clear expected behavior, and similarly straightforward changes.

Before starting work on a new feature, user-visible behavior or default-setting change, CLI or configuration change, terminal or platform support, architectural change, or change spanning multiple components, please open an issue and wait until the intended behavior and scope have been discussed. This helps avoid spending time on an implementation that does not fit the project's direction or expected design.

Agreement on an idea does not guarantee that a particular implementation will be accepted. Implementation details, scope, maintainability, and consistency with the existing application are also considered during review.

Serie is intentionally focused on browsing commit history and its graph. Changes that move it toward a full-featured Git client or introduce substantially more complex TUI interactions require careful discussion and may be considered out of scope.

## Reporting Issues

Before reporting, please check if an issue with the same content already exists.

### Reporting Bugs

Serie renders commit graphs using terminal image display protocols. Before reporting a graph rendering problem, make sure you understand which protocol Serie is using and confirm that every part of your terminal environment supports it. When using a terminal multiplexer, also confirm whether it supports or passes through the selected protocol. See [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html) for the protocols and environments currently supported by Serie.

When reporting a bug, please include the following information:

- Application version
  - `serie --version`
- Installation method
- OS and version
- Terminal emulator and version
- Terminal multiplexer and version, if used
- Git version
- Command-line options, selected image display protocol, and relevant configuration
- Steps to reproduce the problem
- Expected and actual behavior
- Error messages, panic output, and backtrace, if available
- Information about the git repository to reproduce the issue
  - If possible, provide the smallest possible repository or a script that creates it (debugging a repository with 100,000 commits is difficult)
  - If you cannot share the repository, provide relevant output such as `git log --graph --oneline --all`
- For display problems, the terminal size and a screenshot or video

### Suggesting Features

When proposing a feature, describe the problem and workflow it would address, the expected behavior or interaction, and why it fits the project's Goals and Non-Goals. Please also explain why existing configuration or user commands are not sufficient, when applicable, and note any effect on existing behavior or defaults.

### Terminal Emulator Compatibility

Support for an image display protocol does not guarantee that every terminal implements every part of that protocol in the same way. Please verify the actual behavior in the target terminal rather than relying only on its documented protocol support.

For information on tested terminal emulators, refer to [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html).

When reporting compatibility results or proposing a compatibility change, include the terminal and version, OS, multiplexer if any, selected protocol, and the behavior of graph rendering, scrolling, resizing, and opening other views such as commit details.

## Pull Requests

We welcome pull requests, but please note that they are not guaranteed to be accepted. Following these guidelines will increase the likelihood of your pull request being approved.

### Creating Pull Requests

- When creating a pull request, please ensure you follow the same guidelines as [mentioned for issues](#reporting-issues).
- An issue is not required for the straightforward changes described in [Before You Start](#before-you-start).
- Keep each pull request focused on one purpose. Do not include fixes, refactoring, or cleanup that are not directly related to that purpose.
- If the change was discussed in an issue, link the issue and keep the implementation within the agreed scope.
- Preserve existing behavior and defaults unless a change has been discussed and agreed on.
- Describe the problem, the chosen approach, and any important alternatives or tradeoffs.
- Include screenshots or videos for visible UI changes.
- For terminal or platform compatibility changes, describe the environments and operations that were tested.
- Update user documentation when behavior, command-line options, configuration, or keybindings change.
- Update `config.schema.json`, `assets/default-keybind.toml`, and existing tests when they are affected by the change.

### Continuous Integration

We use [GitHub Actions](https://github.com/lusingander/serie/blob/master/.github/workflows/build.yml) to build and test with both stable Rust and the minimum supported Rust version specified by `rust-version` in `Cargo.toml`.

Before submitting a pull request, run the following checks locally:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

If a check cannot be run in your environment, explain that in the pull request.

### Improving the Commit Graph

Improvements to the commit graph are welcome.

Tests for the commit graph are conducted in [./src/tests/graph.rs](./src/tests/graph.rs).

Running the tests will output images and the test repository to `./out/graph`.
If you add new test cases, please add these images under `./tests/graph/`.
If existing graphs are modified, overwrite the images and ensure no unexpected changes have occurred.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, contributors agree to abide by the terms of the applicable license.
