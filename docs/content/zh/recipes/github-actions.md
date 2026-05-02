+++
title = "GitHub Actions 实用方案"
description = "在每次 push 和 pull request 上运行格式化、lint 和静态分析。"
nav_order = 50
nav_section = "实用方案"
+++
# GitHub Actions 实用方案

一个简单的工作流,在每次 push 和 pull request 上运行格式化器、linter 和分析器,并附带原生 PR 注解。

## 快速配置

创建 `.github/workflows/mago.yml`:

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
      - name: Checkout
        uses: actions/checkout@v4

      - name: Set up PHP
        uses: shivammathur/setup-php@v2
        with:
          php-version: "8.4"
          coverage: none
          tools: composer
        env:
          COMPOSER_ALLOW_SUPERUSER: 1

      - name: Install Composer dependencies
        run: composer install --prefer-dist --no-progress

      - name: Set up Mago
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

关于结构的几点说明:

- 把 `format`、`lint` 和 `analyze` 拆成独立步骤,可以在某个步骤失败时仍呈现其余两步的结果。把它们合并到单个 `run:` 中会在第一个失败处短路,导致后面的输出被隐藏。
- `if: success() || failure()` 会在任务未被取消时运行该步骤,这正是我们想要的。`always()` 在配置阶段失败后也会运行该步骤。
- 使用 `mago format --check`,而不是 `--dry-run`。`--check` 在有文件需要格式化时以非零状态退出;`--dry-run` 仅打印 diff,始终以零状态退出。
- Mago 会通过 `GITHUB_ACTIONS` 环境变量识别 GitHub Actions,自动切换到 `--reporting-format=github`,产出原生 PR 注解。无需额外配置。在 1.17.0 及更早版本中,你需要为 `mago lint` 和 `mago analyze` 手动传入 `--reporting-format=github`。

## 使用 Docker 镜像

如果你不想在 runner 上安装 Mago,可以把 [官方 Docker 镜像](/recipes/docker/) 当作容器作业来运行:

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
        if: success() || failure()
        run: mago lint

      - name: Analyze
        if: success() || failure()
        run: mago analyze
```

镜像不包含 PHP 或 Composer。对格式化器和 linter 来说没问题,但分析器需要 Composer 依赖才能解析符号。如果要做静态分析,优先选择 [setup-mago 方案](#quick-setup),先运行 `composer install`。
