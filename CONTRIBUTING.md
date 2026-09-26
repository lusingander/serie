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

When reporting a bug, please include the following information:

- Application version
  - `serie --version`
- Version of the terminal emulator and the OS it's running on
- Information about the git repository to reproduce the issue
  - If possible, provide the smallest possible repository (debugging a repository with 100,000 commits is difficult)

### Suggesting Features

When proposing a feature, describe the problem and workflow it would address, the expected behavior or interaction, and why it fits the project's Goals and Non-Goals. Please also explain why existing configuration or user commands are not sufficient, when applicable, and note any effect on existing behavior or defaults.

### Terminal Emulator Compatibility

If the application does not work with your terminal emulator, please first check whether the terminal emulator supports the target image display protocol.

For information on tested terminal emulators, refer to [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html).

## Pull Requests

We welcome pull requests, but please note that they are not guaranteed to be accepted. Following these guidelines will increase the likelihood of your pull request being approved.

### Creating Pull Requests

- When creating a pull request, please ensure you follow the same guidelines as [mentioned for issues](#reporting-issues).
- An issue is not required for the straightforward changes described in [Before You Start](#before-you-start).
- Keep each pull request focused on one purpose. Do not include fixes, refactoring, or cleanup that are not directly related to that purpose.
- If the change was discussed in an issue, link the issue and keep the implementation within the agreed scope.
- Preserve existing behavior and defaults unless a change has been discussed and agreed on.

### Continuous Integration

We use [GitHub Actions](https://github.com/lusingander/serie/blob/master/.github/workflows/build.yml) to perform basic checks:

- Run both stable and MSRV versions of Rust.
- Run build, test, format, and lint.

### Improving the Commit Graph

Improvements to the commit graph are welcome.

Tests for the commit graph are conducted in [./src/tests/graph.rs](./src/tests/graph.rs).

Running the tests will output images and the test repository to `./out/graph`.
If you add new test cases, please add these images under `./tests/graph/`.
If existing graphs are modified, overwrite the images and ensure no unexpected changes have occurred.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, contributors agree to abide by the terms of the applicable license.
