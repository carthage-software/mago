# Contributing to Mago

Thank you for your interest in contributing to Mago, the Oxidized PHP Toolchain! Whether you're fixing bugs, improving documentation, or proposing new features, your help is invaluable.

## Code of Conduct

The code of conduct is described in [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)

## Issues

We use GitHub issues to track issues within Mago.

Please ensure your description is clear and has sufficient instructions to be able to reproduce the issue.

## Getting started

Contributing to open-source can be scary. Don't be afraid!
We are looking forward working together to improve this package!

Here is a small checklist to get you going:

1. **Discuss the changes**:
   Open an issue or comment on an existing one to discuss the changes you plan to make.

2. **Fork the repository**:
   Fork this repository to your own GitHub account.

3. **Clone the repository**:
   Clone the repository to your local machine:

   ```bash
   git clone https://github.com/<your-username>/mago.git
   ```

4. **Set up your environment**:

   - Install [Rust](https://www.rust-lang.org/tools/install)
   - Install [Just](https://github.com/casey/just)
   - Run `just build` to set up the project.
   - If you use [Nix](https://nixos.org): Run `nix develop` and `just build`.

5. **Create a branch**:
   Create a new branch with a descriptive name:

   ```bash
   git checkout -b <branch-name>
   ```

6. **Make your changes**:
   Implement the changes and follow the coding guidelines.

7. **Verify your changes**:
   Run the tests to make sure your changes are correct:

   ```bash
   just test
   ```

   Lint your code to ensure it follows the coding standards:

   ```bash
   just lint
   ```

8. **Commit your changes**:
   Commit your changes and write a descriptive commit message:

   ```bash
   git commit -am "Your message here"
   ```

9. **Push your changes**:
   Push your changes to your fork:

   ```bash
   git push origin <branch-name>
   ```

10. **Submit a Pull Request**:
    Submit a Pull Request to the main repository.

    - Go to the [main repository](https://github.com/carthage-software/mago)
    - Click on the "New Pull Request" button
    - Select your fork and branch
    - Write a descriptive title and message
    - Click on "Create Pull Request"

## Submitting Pull Requests

Before we can merge your pull request, please follow these guidelines to maintain code quality and consistency:

### Tests

If you are submitting a bug-fix, please add a test case to reproduce the bug.
If you are submitting a new feature, please make sure to add tests for all possible code paths.

To run the tests, use `just test`.

### Coding Standards

Ensure your code follows the coding standards and conventions used in the project.

- Run `just lint` to check your code for style issues.
- Run `just fix` to automatically fix style issues.

### License

By contributing to Mago, you agree that your contributions will be licensed under the dual MIT/Apache-2.0 license, consistent with the repository's [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE) files.

## Security Disclosures

You can read more about how to report security issues in our [Security Policy](./SECURITY.md).
