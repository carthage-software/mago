---
title: Docker recipe
outline: deep
---

# 🐳 Docker recipe

Run **Mago** in any environment without installing it locally. The official container image is built from `scratch` with a statically-linked binary, resulting in a minimal image (~26 MB) with no OS overhead.

## Image

The official image is published to the GitHub Container Registry:

```
ghcr.io/carthage-software/mago
```

## Available tags

Each release produces multiple tags for flexible version pinning:

| Tag               | Example                                 | Description                                      |
| :---------------- | :-------------------------------------- | :----------------------------------------------- |
| `latest`          | `ghcr.io/carthage-software/mago:latest` | Always points to the newest release              |
| `<version>`       | `ghcr.io/carthage-software/mago:1.13.0` | Pinned to an exact version                       |
| `<major>.<minor>` | `ghcr.io/carthage-software/mago:1.13`   | Tracks the latest patch within a minor version   |
| `<major>`         | `ghcr.io/carthage-software/mago:1`      | Tracks the latest release within a major version |

The image supports both `linux/amd64` and `linux/arm64` architectures. Docker will automatically pull the correct variant for your platform.

## Quick start

Mount your project directory and run any Mago command:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

## Usage examples

### Linting

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

### Formatting

Check for formatting issues without modifying files:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt --check
```

Apply formatting fixes:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt
```

### Static analysis

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago analyze
```

### Checking the version

```sh
docker run --rm ghcr.io/carthage-software/mago --version
```

## Using Docker in CI/CD

### GitHub Actions

You can use the Docker image directly in GitHub Actions as a container job:

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

      - name: Check Formatting
        run: mago fmt --check

      - name: Lint
        run: mago lint --reporting-format=github

      - name: Analyze
        run: mago analyze --reporting-format=github
```

:::warning
The Docker image does not include PHP or Composer. This works well for formatting and linting, but the **analyzer** needs access to your project's Composer dependencies to resolve symbols correctly. If your project depends on third-party packages, running `mago analyze` without installed dependencies will produce false positives for undefined symbols. For analysis, consider using a [native installation](/guide/installation) with Composer dependencies installed.
:::

### GitLab CI

```yaml
mago:
  image: ghcr.io/carthage-software/mago:1
  script:
    - mago fmt --check
    - mago lint
    - mago analyze
```

### Bitbucket Pipelines

```yaml
pipelines:
  default:
    - step:
        name: Mago Code Quality
        image: ghcr.io/carthage-software/mago:1
        script:
          - mago fmt --check
          - mago lint
          - mago analyze
```

## Shell alias

For convenience, you can create a shell alias to use the Docker image as if Mago were installed locally:

```sh
alias mago='docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago:1'
```

Add this to your `~/.bashrc`, `~/.zshrc`, or equivalent shell configuration file. After reloading your shell, you can use `mago` as usual:

```sh
mago lint
mago fmt --check
mago analyze
```

## Limitations

The container image is built from `scratch` and contains only the Mago binary. This means:

- **No shell**: You cannot exec into the container or run shell commands inside it.
- **No git**: The `--staged` flag for `lint` and `fmt` commands will not work, as there is no `git` binary available inside the container.

For workflows that require `--staged` support, use a [native installation](/guide/installation).
