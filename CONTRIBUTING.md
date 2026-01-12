# Contribution Guide

Thank you for considering contributing. Please review the guidelines below before making a contribution.

To ensure that your contributions are considered, please follow this guidelines. Contributions that do not adhere to these guidelines may not be accepted.

## Reporting Issues

Before reporting, please check if an issue with the same content already exists.

Also, please refer to [FAQ](https://lusingander.github.io/serie/faq/index.html).

### Reporting Bugs

When reporting a bug, please include the following information:

- Application version
  - `serie --version`
- Version of the terminal emulator and the OS it's running on
- Information about the git repository to reproduce the issue
  - If possible, provide the smallest possible repository (debugging a repository with 100,000 commits is difficult)

### Suggesting Features

Before proposing a new feature, please review the [Goals](https://lusingander.github.io/serie/introduction/index.html#goals) and [Non-Goals](https://lusingander.github.io/serie/introduction/index.html#non-goals).

### Terminal Emulator Compatibility

If the application does not work with your terminal emulator, please first check whether the terminal emulator supports the target image display protocol.

For information on tested terminal emulators, refer to [Compatibility](https://lusingander.github.io/serie/getting-started/compatibility.html).

## Pull Requests

We welcome pull requests, but please note that they are not guaranteed to be accepted. Following this guideline will increase the likelihood of your pull request being approved.

### Creating pull requests

- When creating a pull request, please ensure you follow the same guidelines as [mentioned for issues](#reporting-issues).
- An issue is not required for every pull request. For small or straightforward changes (such as documentation fixes or obvious bug fixes), feel free to open a pull request directly.
- For more complex changes or behavior-altering fixes, opening an issue first is strongly recommended to discuss the approach and avoid unnecessary rework.
- Do not include fixes that are not directly related to the pull request topic.

### Continuous Integration

We use [GitHub Actions](https://github.com/lusingander/serie/blob/master/.github/workflows/build.yml) to perform basic checks:

- Run both stable and MSRV versions of Rust.
- Run build, test, format, and lint.

### Improving the Commit Graph

Improvements to the commit graph are welcome.

Tests for the commit graph are conducted in [./tests/graph.rs](./tests/graph.rs).

Running the tests will output images and the test repository to `./out/graph`.
If you add new test cases, please add these images under `./tests/graph/`.
If existing graphs are modified, overwrite the images and ensure no unexpected changes have occurred.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, contributors agree to abide by the terms of the applicable license.
