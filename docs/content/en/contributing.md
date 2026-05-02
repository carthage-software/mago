+++
title = "Contributing"
description = "How to set up a local Mago checkout, run the tests, and submit a change."
nav_order = 30
nav_section = "Reference"
+++
# Contributing to Mago

Thanks for considering a contribution. The steps below get you from a clean checkout to a pull request.

## Getting started

1. Open an issue or comment on an existing one before starting on anything non-trivial. It is the easiest way to make sure your work lines up with the project's direction.

2. Fork the repository on GitHub and clone your fork:

   ```bash
   git clone https://github.com/<your-username>/mago.git
   ```

3. Install [Rust](https://www.rust-lang.org/tools/install) and [Just](https://github.com/casey/just), then run `just build` to set up the project. Nix users can run `nix develop` first and then `just build`.

4. Create a branch:

   ```bash
   git checkout -b feature/my-awesome-change
   ```

5. Make your changes following the project's coding style.

6. Run the tests and linter:

   ```bash
   just test
   just check
   ```

7. Commit and push:

   ```bash
   git commit -m "feat: add my awesome change"
   git push origin feature/my-awesome-change
   ```

8. Open a pull request against the [main repository](https://github.com/carthage-software/mago).

## Pull requests

Bug fixes should include a test that reproduces the bug. New features should include comprehensive coverage. By contributing, you agree that your contributions are licensed under the project's dual MIT / Apache-2.0 license.

To report a security issue, follow the steps in the [security policy](https://github.com/carthage-software/mago/security/policy).
