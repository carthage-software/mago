+++
title = "报告与修复选项"
description = "lint、analyze 和 ast 共用的一组参数,用于报告问题、应用修复以及管理 baseline。"
nav_order = 40
nav_section = "基础"
+++
# 报告与修复选项

`mago lint`、`mago analyze` 和 `mago ast` 共用一组参数,用来控制问题如何被报告、修复如何被应用。本页是这些参数的中心参考,这样我们就不必在每个命令页里重复它们。

## 自动修复

大多数 linter 规则以及若干分析器检查都附带了自动修复。下列参数控制如何应用修复,以及哪些类别可被修复。

| 参数 | 说明 |
| :--- | :--- |
| `--fix` | 对发现的问题应用所有安全修复。 |
| `--fixable-only`, `-f` | 把输出过滤到具有自动修复的问题上。 |
| `--unsafe` | 应用被标记为 unsafe 的修复。这些修复可能改变行为,需要审阅。 |
| `--potentially-unsafe` | 应用被标记为 potentially-unsafe 的修复。比 unsafe 风险低,但仍值得简单审阅。 |
| `--format-after-fix`, `fmt` | 在每个被 `--fix` 修改过的文件上运行格式化器。 |
| `--dry-run`, `-d`, `diff` | 以统一 diff 的形式预览修复,不写入任何内容。 |

## 报告

Mago 如何呈现它发现的问题。

| 参数 | 说明 |
| :--- | :--- |
| `--sort` | 按级别、code、位置依次排序报告中的问题。 |
| `--reporting-target <TARGET>` | 报告写入的目标。可选值:`stdout`(默认)、`stderr`。 |
| `--reporting-format <FORMAT>` | 输出格式。详见下文;默认自动检测。 |
| `--minimum-fail-level <LEVEL>`, `-m` | 触发非零退出的最低级别。可选值:`note`、`help`、`warning`、`error`。默认采用配置文件中的值,若无则为 `error`。 |
| `--minimum-report-level <LEVEL>` | 报告中包含的最低级别。低于此值的问题在打印前被过滤掉。 |
| `--retain-code <CODE>` | 仅保留指定 code 的问题。这是报告过滤,不是执行过滤。可重复使用。 |

`--retain-code` 与 `--only`(只有 `mago lint` 接受)不同:

- `mago lint --only <RULE>` 仅运行指定的规则。其他规则被完全跳过,因此更快。
- `mago lint --retain-code <CODE>` 运行所有规则,然后把输出过滤到你列出的 code。

```sh
mago lint --only no-unused-variable                                  # only run that rule
mago lint --retain-code no-unused-variable                           # run everything, show only this code
mago lint --retain-code no-unused-variable --retain-code semantics   # multiple codes
mago analyze --retain-code invalid-argument --retain-code type-mismatch
```

当你想对某条具体规则做快速反馈时,使用 `--only`。当你想全面覆盖但聚焦报告时,使用 `--retain-code`。

### 报告格式

用 `--reporting-format` 显式选择:

- 人类可读:`rich`、`medium`、`short`、`ariadne`、`emacs`。
- CI / 机器可读:`github`、`gitlab`、`json`、`checkstyle`、`sarif`。
- 摘要:`count`、`code-count`。

### 自动检测

未设置 `--reporting-format` 时,Mago 会根据环境选择一种格式:

| 环境 | 通过什么检测 | 默认格式 |
| :--- | :--- | :--- |
| GitHub Actions | `GITHUB_ACTIONS` | `github` |
| GitLab CI | `GITLAB_CI` | `gitlab` |
| AI 编码助手 | `CLAUDECODE`、`GEMINI_CLI`、`CODEX_SANDBOX`、`OPENCODE_CLIENT` | `medium` |
| 其他 | (无) | `rich` |

因此 CI 流水线得到原生注解,AI 助手得到一种节省 token 的格式,不需要任何配置。显式传入 `--reporting-format` 可覆盖。

> 自动检测自 Mago 1.18 起可用。在 1.17 及更早版本中,请显式设置 `--reporting-format=github` 或 `--reporting-format=gitlab`。

## Baseline

用于管理 baseline 文件的参数。完整指南在 [baseline 页面](/fundamentals/baseline/)。

| 参数 | 说明 |
| :--- | :--- |
| `--generate-baseline` | 生成一份新的 baseline 文件,记录当前所有问题。 |
| `--baseline <PATH>` | 使用给定路径的 baseline。 |
| `--backup-baseline` | 重新生成时,在覆盖前把旧 baseline 复制为 `<file>.bkp`。 |
| `--ignore-baseline` | 忽略任何已配置或指定的 baseline,报告所有问题。 |
