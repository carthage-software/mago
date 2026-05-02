+++
title = "Pre-commit hooks recipe"
description = "Run lint, analyze, and format on staged files before every commit."
nav_order = 40
nav_section = "Recipes"
+++
# Pre-commit hooks recipe

Run Mago automatically before every git commit. The examples below all act on the staged files only, so the hook stays fast even on large repositories.

## Quick setup

Create `.git/hooks/pre-commit` and make it executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Hook configurations

Pick the setup that matches your workflow.

### Auto-format staged files

Format staged PHP files and re-stage the formatted versions. The smoothest experience for developers; nothing to remember.

```bash
#!/bin/sh

mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

`--staged` finds staged files and only processes those. For `fmt --staged`, formatted files are re-staged automatically. For `lint --staged` and `analyze --staged`, when combined with `--fix`, fixed files are re-staged.

### Auto-fix and auto-format staged files

This adds `--fix` to the lint step. `--fail-on-remaining` blocks the commit if any issues could not be auto-fixed and still need manual attention. Without it, `--fix` exits zero even when unfixed issues remain.

```bash
#!/bin/sh

mago lint --fix --fail-on-remaining --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the remaining issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --staged
if [ $? -ne 0 ]; then
    echo "Formatting failed. Please check the error above."
    exit 1
fi

exit 0
```

For more aggressive fixes, use `--fix --unsafe` or `--fix --potentially-unsafe`:

```bash
mago lint --fix --potentially-unsafe --fail-on-remaining --staged
```

### Block commits when formatting drifts

Refuse the commit if any staged file is not properly formatted, requiring the developer to format manually.

```bash
#!/bin/sh

mago lint --staged
if [ $? -ne 0 ]; then
    echo "Linting failed. Please fix the issues before committing."
    exit 1
fi

mago analyze --staged
if [ $? -ne 0 ]; then
    echo "Static analysis failed. Please fix the issues before committing."
    exit 1
fi

mago fmt --check
if [ $? -ne 0 ]; then
    echo "Some files are not formatted. Please run 'mago fmt' before committing."
    exit 1
fi

exit 0
```

## Husky

If you use [Husky](https://typicode.github.io/husky/), add the commands to `.husky/pre-commit`:

```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

mago lint --staged
mago analyze --staged
mago fmt --staged
```

## CaptainHook

If you use [CaptainHook](https://docs.captainhook.info/), add the actions to `captainhook.json`:

```json
{
    "pre-commit": {
        "enabled": true,
        "actions": [
            { "action": "mago lint --staged" },
            { "action": "mago analyze --staged" },
            { "action": "mago fmt --staged" }
        ]
    }
}
```

For the check-only variant, swap the last action for `mago fmt --check`.

## `--staged` versus `--check`

| Aspect | `--staged` | `--check` |
| :--- | :--- | :--- |
| Behaviour | Formats staged files and re-stages them. | Reports unformatted files; fails if any. |
| Developer action | None. | Must run `mago fmt` manually if the check fails. |
| Best for | Teams that want seamless formatting. | Teams that want explicit control over changes. |
| Partial staging | Formats the staged content, leaves the working tree alone. | Works regardless of staging state. |
