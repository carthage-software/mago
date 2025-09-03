---
title: Contributing to Mago
---

Thank you for your interest in contributing to **Mago**! We're excited to build the future of PHP tooling with you. Whether you're fixing bugs, improving documentation, or proposing new features, your help is invaluable.

---

## Getting started

Contributing to open-source can be intimidating, but don't worry! We're here to help you get started. Here is a small checklist to get you going:

1.  **Discuss first**. Before you start coding, please open an issue or comment on an existing one to discuss the changes you plan to make. This helps ensure your work aligns with the project's goals.

2.  **Fork & clone**. Fork the repository to your own GitHub account and clone it to your local machine:

    ```bash
    git clone https://github.com/<your-username>/mago.git
    ```

3.  **Set up your environment:**
    - Install [Rust](https://www.rust-lang.org/tools/install)
    - Install [Just](https://github.com/casey/just)
    - Run `just build` to set up the project and install dependencies.
    - If you use [Nix](https://nixos.org): Run `nix develop` and `just build`.

4.  **Create a branch**. Create a new branch with a descriptive name:

    ```bash
    git checkout -b feature/my-awesome-change
    ```

5.  **Make your changes**. Implement your changes and follow the coding guidelines.

6.  **Verify your changes**. Run the tests and linter to make sure everything is correct and follows our coding standards:

    ```bash
    # Run all tests
    just test

    # Check for linting issues
    just lint
    ```

7.  **Commit and push**. Commit your changes with a descriptive message and push them to your fork:

    ```bash
    git commit -m "feat: add my awesome change"
    git push origin feature/my-awesome-change
    ```

8.  **Submit a pull request**. Go to the main [Mago repository](https://github.com/carthage-software/mago) and open a new pull request with your changes.

---

## Submitting pull requests

If you're fixing a bug, please add a test case that reproduces it. If you're adding a new feature, ensure it has comprehensive test coverage.

By contributing, you agree that your contributions will be licensed under the dual MIT/Apache-2.0 license.

To report a security vulnerability, please follow the instructions in our [Security Policy](https://github.com/carthage-software/mago/security/policy).
