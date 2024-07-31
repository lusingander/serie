# Contribution Guide

Thank you for considering contributing. Please review the guidelines below before making a contribution.

## Reporting Issues

Before reporting, please check if an issue with the same content already exists.

### Reporting Bugs

When reporting a bug, please include the following information:

- Application version
- Terminal emulator and version being used
- Information about the git repository to reproduce the issue
  - If possible, provide the smallest possible repository (debugging a repository with 100,000 commits is difficult)

### Suggesting Features

Before proposing a new feature, please review the [Goals](./README.md#goals) and [Non-Goals](./README.md#non-goals).

### Terminal Emulator Compatibility

If the application does not work with your terminal emulator, please check whether the terminal emulator supports the target image display protocol.

For information on tested terminal emulators, refer to [Compatibility](./README.md#compatibility).

We welcome contributions of information on new terminal emulators. Please share them on the [Discussions](https://github.com/lusingander/serie/discussions).

## Pull Requests

Creating a pull request does not necessarily require an issue. For complex problems, creating an issue beforehand might make the process smoother.

### Improving the Commit Graph

Improvements to the commit graph are welcome.

Tests for the commit graph are conducted in [./tests/graph.rs](./tests/graph.rs).

Running the tests will output images and the test repository to `./out/graph`.
If you add new test cases, please add these images under `./tests/graph/`.
If existing graphs are modified, overwrite the images and ensure no unexpected changes have occurred.

## Additional Information

If you have any questions or concerns, please use the [Discussions](https://github.com/lusingander/serie/discussions).
