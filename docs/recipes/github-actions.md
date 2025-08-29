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

      - name: Run Mago
        run: |
          mago format --dry-run
          mago lint --reporting-format=github
          mago analyze --reporting-format=github
```
