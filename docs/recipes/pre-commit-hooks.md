---
title: Pre-commit hooks recipe
---

# Pre-commit hooks recipe

Ensure code quality before every commit by integrating **Mago** into your git pre-commit hooks. This guide shows you how to set up hooks that run linting, static analysis, and formatting checks automatically.

## Quick setup

Create a file at `.git/hooks/pre-commit` (or add to your existing hook) and make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Hook configurations

Choose the configuration that best fits your workflow:

### Option 1: Auto-format staged files

This approach automatically formats your staged PHP files and includes the formatted versions in your commit. This is the most seamless developer experience.

```bash
#!/bin/sh

# Lint only staged files
mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

# Analyze only staged files
mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

# Format staged files and re-stage them
mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

The `--staged` flag on all three commands:

- Finds all files currently staged for commit
- Only processes those files instead of the entire project
- For `fmt --staged`, automatically re-stages formatted files
- For `lint --staged` and `analyze --staged`, when combined with `--fix`, automatically re-stages fixed files

:::warning
The `fmt --staged` flag will fail if a staged file also has unstaged changes. This prevents accidentally including unintended changes in your commit. Either stage all changes or stash the unstaged ones before committing.
:::

### Option 2: Auto-fix and auto-format staged files

This approach automatically fixes linting issues in staged files, then formats them. Fixed and formatted files are re-staged automatically.

```bash
#!/bin/sh

# Fix linting issues in staged files and re-stage them
mago lint --fix --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the remaining issues before committing."
    exit 1
fi

# Analyze only staged files
mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

# Format staged files and re-stage them
mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

You can also use `--fix --unsafe` or `--fix --potentially-unsafe` to apply more aggressive fixes:

```bash
mago lint --fix --potentially-unsafe --staged
```

### Option 3: Block commits if not formatted

This approach blocks the commit if any staged files aren't properly formatted, requiring developers to format their code before committing.

```bash
#!/bin/sh

# Lint only staged files
mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

# Analyze only staged files
mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

# Check if staged files are formatted (without modifying them)
mago fmt --check
if [ $? -ne 0 ]; then
    echo "Some files are not formatted. Please run 'mago fmt' before committing."
    exit 1
fi

exit 0
```

## Using with Husky (Node.js projects)

If your project uses [Husky](https://typicode.github.io/husky/) for git hooks, add these commands to your `.husky/pre-commit` file:

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

mago lint --staged
mago analyze --staged
mago fmt --staged  # or: mago fmt --check
```

## Using with CaptainHook (PHP projects)

If your project uses [CaptainHook](https://docs.captainhook.info/), add the following to your `captainhook.json`:

```json
{
  "pre-commit": {
    "enabled": true,
    "actions": [
      {
        "action": "mago lint --staged"
      },
      {
        "action": "mago analyze --staged"
      },
      {
        "action": "mago fmt --staged"
      }
    ]
  }
}
```

Or if you prefer the check-only approach:

```json
{
  "pre-commit": {
    "enabled": true,
    "actions": [
      {
        "action": "mago lint --staged"
      },
      {
        "action": "mago analyze --staged"
      },
      {
        "action": "mago fmt --check"
      }
    ]
  }
}
```

## Comparison: `--staged` vs `--check`

| Aspect               | `--staged`                                         | `--check`                                     |
| -------------------- | -------------------------------------------------- | --------------------------------------------- |
| **Behavior**         | Auto-formats staged files and re-stages them       | Checks if files are formatted, fails if not   |
| **Developer action** | None required                                      | Must run `mago fmt` manually if check fails   |
| **Best for**         | Teams that want seamless formatting                | Teams that want explicit control over changes |
| **Partial staging**  | Fails if file has both staged and unstaged changes | Works regardless of staging state             |
