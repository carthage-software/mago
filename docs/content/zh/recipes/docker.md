+++
title = "Docker 实用方案"
description = "在任何环境中运行 Mago,而无需在本地安装。"
nav_order = 60
nav_section = "实用方案"
+++
# Docker 实用方案

官方容器镜像基于 `scratch` 构建,使用静态链接的二进制,因此镜像很小(约 26 MB)且不包含任何操作系统。

## 镜像

镜像位于 GitHub Container Registry 上的 `ghcr.io/carthage-software/mago`。

## Tag

每次发布会发布多个 tag,你可以按所需精度进行锁定:

| Tag | 示例 | 说明 |
| :--- | :--- | :--- |
| `latest` | `ghcr.io/carthage-software/mago:latest` | 始终指向最新发布。 |
| `<version>` | `ghcr.io/carthage-software/mago:1.25.1` | 锁定到精确版本。 |
| `<major>.<minor>` | `ghcr.io/carthage-software/mago:1.25` | 跟随某个次版本下的最新补丁。 |
| `<major>` | `ghcr.io/carthage-software/mago:1` | 跟随某个主版本下的最新发布。 |

镜像支持 `linux/amd64` 和 `linux/arm64`。Docker 会为你的宿主机拉取对应变体。

## 快速开始

挂载你的项目目录,运行任意命令:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

## 示例

Lint:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

只检查格式不写入:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt --check
```

应用格式化:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt
```

运行静态分析:

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago analyze
```

打印版本:

```sh
docker run --rm ghcr.io/carthage-software/mago --version
```

## CI 集成

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

镜像不包含 PHP 或 Composer。这对格式化器和 linter 来说没有问题。分析器需要安装项目的 Composer 依赖才能正确解析符号;否则会对未定义符号产生误报。如果你的项目依赖第三方包并打算运行分析器,优先选择 [原生安装](/guide/installation/) 并安装好 Composer 依赖。

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

## Shell 别名

把镜像当作本地二进制来用:

```sh
alias mago='docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago:1'
```

把这一行加入 shell 初始化文件,重新加载 shell,然后就可以像往常一样运行 `mago lint`(或任何其他子命令)。
