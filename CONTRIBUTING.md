# Contribution Guide

Thank you for considering contributing. Please review the guidelines below before making a contribution.

To ensure that your contributions are considered, please follow this guidelines. Contributions that do not adhere to these guidelines may not be accepted.

## Reporting Issues

Before reporting, please check if an issue with the same content already exists.

### Reporting Bugs

When reporting a bug, please include the following information:

- Application version
- Version of the terminal emulator and the OS it's running on
- Information about the git repository to reproduce the issue
  - If possible, provide the smallest possible repository (debugging a repository with 100,000 commits is difficult)

### Suggesting Features

Before proposing a new feature, please review the [Goals](./README.md#goals) and [Non-Goals](./README.md#non-goals).

### Terminal Emulator Compatibility

If the application does not work with your terminal emulator, please first check whether the terminal emulator supports the target image display protocol.

For information on tested terminal emulators, refer to [Compatibility](./README.md#compatibility).

Please share your experience with other terminals on the [Discussions](https://github.com/lusingander/serie/discussions/29). Please share any necessary information listed at the top of the Discussions.

## Pull Requests

We welcome pull requests, but please note that they are not guaranteed to be accepted. Following this guideline will increase the likelihood of your pull request being approved.

### Creating pull requests

- When creating a pull request, please ensure you follow the same guidelines as [mentioned for issues](#reporting-issues).
- Creating a pull request does not necessarily require an issue. But if the problem is complex, creating an issue beforehand might make the process smoother.
- Do not include fixes that are not directly related to the pull request topic.

### Improving the Commit Graph

Improvements to the commit graph are welcome.

Tests for the commit graph are conducted in [./tests/graph.rs](./tests/graph.rs).

Running the tests will output images and the test repository to `./out/graph`.
If you add new test cases, please add these images under `./tests/graph/`.
If existing graphs are modified, overwrite the images and ensure no unexpected changes have occurred.

## License

This project is licensed under the [MIT License](LICENSE). By contributing, contributors agree to abide by the terms of the applicable license.

## Additional Information

If you have any questions or concerns, please use the [Discussions](https://github.com/lusingander/serie/discussions).
