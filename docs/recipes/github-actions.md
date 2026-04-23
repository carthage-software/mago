---
title: GitHub Actions recipe
---

# 🧩 GitHub Actions recipe

Automate your code quality checks by running **Mago** directly in your GitHub workflow. This setup will check for formatting and linting errors on every push and pull request, providing direct feedback within GitHub.

## Quick setup

Create a new file at `.github/workflows/mago.yml` and add the following content:

```yaml
name: Mago Code Quality

on:
  push:
  pull_request:

jobs:
  mago:
    name: Run Mago Checks
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Setup PHP with Composer cache
        uses: shivammathur/setup-php@v2
        with:
          php-version: "8.4" # Or your project's version
          coverage: none
          tools: composer
        env:
          COMPOSER_ALLOW_SUPERUSER: 1

      - name: Install Composer Dependencies
        run: composer install --prefer-dist --no-progress

      - name: Setup Mago
        uses: nhedger/setup-mago@v1

      - name: Check formatting
        run: mago format --check

      - name: Lint
        if: success() || failure()
        run: mago lint

      - name: Analyze
        if: success() || failure()
        run: mago analyze
```

:::tip
Splitting `format`, `lint`, and `analyze` into separate steps (and using `if: success() || failure()` on the later ones) lets a single run surface findings from all three tools even when an earlier one fails. A combined `run:` block short-circuits on the first failure, hiding findings from the later tools. `success() || failure()` runs the step as long as the job was not cancelled, unlike `always()` which would also run it after a failure in job setup. Use `mago format --check` (not `--dry-run`) so the step actually fails CI when code is not formatted — `--dry-run` only prints a diff and always exits `0`.
:::

:::tip
Mago automatically detects GitHub Actions via the `GITHUB_ACTIONS` environment variable and defaults to `--reporting-format=github`, producing native PR annotations. No extra configuration needed.
:::

:::warning
If you are using Mago 1.17.0 or earlier, you must explicitly pass `--reporting-format=github` to `mago lint` and `mago analyze` for GitHub Actions annotations. Auto-detection was introduced in a later release.
:::

## Using the Docker image

If you prefer not to install Mago on the runner, you can use the official [Docker image](/recipes/docker) as a container job:

```yaml
name: Mago Code Quality

on:
  push:
  pull_request:

jobs:
  mago:
    name: Run Mago Checks
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/carthage-software/mago:1
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Check formatting
        run: mago fmt --check

      - name: Lint
        if: success() || failure()
        run: mago lint

      - name: Analyze
        if: success() || failure()
        run: mago analyze
```

:::warning
The Docker image does not include PHP or Composer. This works well for formatting and linting, but the **analyzer** needs access to your project's Composer dependencies to resolve symbols correctly. If your project depends on third-party packages, running `mago analyze` without installed dependencies will produce false positives for undefined symbols. For analysis, prefer the [setup-mago approach](#quick-setup) with Composer dependencies installed.
:::
