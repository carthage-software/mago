+++
title = "Docker recipe"
description = "Run Mago in any environment without installing it locally."
nav_order = 60
nav_section = "Recipes"
+++
# Docker recipe

The official container image is built from `scratch` with a statically linked binary, so the image is small (around 26 MB) and ships no OS.

## Image

The image lives at `ghcr.io/carthage-software/mago` on the GitHub Container Registry.

## Tags

Each release publishes several tags so you can pin at the precision you want:

| Tag | Example | Description |
| :--- | :--- | :--- |
| `latest` | `ghcr.io/carthage-software/mago:latest` | Always points to the newest release. |
| `<version>` | `ghcr.io/carthage-software/mago:1.25.2` | Pinned to an exact version. |
| `<major>.<minor>` | `ghcr.io/carthage-software/mago:1.25` | Tracks the latest patch within a minor version. |
| `<major>` | `ghcr.io/carthage-software/mago:1` | Tracks the latest release within a major version. |

The image supports `linux/amd64` and `linux/arm64`. Docker pulls the right variant for your host.

## Quick start

Mount your project directory and run any command:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

## Examples

Lint:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

Check formatting without writing:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt --check
```

Apply formatting:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt
```

Run static analysis:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago analyze
```

Print the version:

```sh
docker run --rm ghcr.io/carthage-software/mago --version
```

## CI integration

### GitHub Actions

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
      - name: Checkout
        uses: actions/checkout@v4

      - name: Check formatting
        run: mago fmt --check

      - name: Lint
        run: mago lint --reporting-format=github

      - name: Analyze
        run: mago analyze --reporting-format=github
```

The image does not include PHP or Composer. That works fine for the formatter and linter. The analyzer needs your project's Composer dependencies installed to resolve symbols correctly; without them it will report false positives for undefined symbols. If your project depends on third-party packages and you want to run the analyzer, prefer a [native installation](/guide/installation/) with Composer dependencies installed.

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

Treating the image as if it were a local binary:

```sh
alias mago='docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago:1'
```

Add the line to your shell init file, reload the shell, and run `mago lint` (or any other subcommand) as usual.
