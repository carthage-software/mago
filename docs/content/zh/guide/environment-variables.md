+++
title = "环境变量"
description = "Mago 读取的所有环境变量、各自的作用,以及它们在优先级链中的位置。"
nav_order = 50
nav_section = "指南"
+++
# 环境变量

Mago 读取一小组环境变量。其中一部分用于覆盖 `mago.toml` 中的键,其余用于控制运行时(日志、颜色、配置文件查找)。

## 运行时

### `MAGO_LOG`

日志级别。在排查意外结果时很有用。

可选值:`trace`、`debug`、`info`、`warn`、`error`。

```sh
MAGO_LOG=trace mago lint
```

### `NO_COLOR`

设为任何真值即可关闭所有彩色输出。遵循 [no-color.org](https://no-color.org/) 约定。

```sh
NO_COLOR=1 mago lint
```

### `FORCE_COLOR`

设为任何真值即可在 stdout 不是终端时也强制启用彩色输出。优先级高于 `NO_COLOR`。遵循 [force-color.org](https://force-color.org/) 约定。

```sh
FORCE_COLOR=1 mago lint | less -R
```

### `XDG_CONFIG_HOME`

当不存在项目级配置文件时,Mago 按照 [XDG Base Directory 规范](https://specifications.freedesktop.org/basedir-spec/latest/) 寻找全局配置。回退链是:

1. `$XDG_CONFIG_HOME/mago.toml`(若已设置)。
2. `$HOME/.config/mago.toml`。
3. `$HOME/mago.toml`。

设置 `XDG_CONFIG_HOME` 会改变第一个查找目录。

```sh
XDG_CONFIG_HOME=/path/to/config mago lint
```

## 保留前缀 `MAGO_`

Mago 为自身保留 `MAGO_` 前缀。只有本页文档化的变量才被官方识别。其他以 `MAGO_` 为前缀的变量被保留作内部用途,可能会在未来的版本中被默默忽略或重新使用。

> 早期版本会自动把每个 `MAGO_*` 变量映射到配置树中,因此像 `MAGO_LINT=1` 这样的设置会因 "unknown field" 错误而崩溃。Mago 1.25 把它收窄为下面这个明确的列表。

## 配置覆盖

下列变量会覆盖 `mago.toml` 中对应的键。它们仅覆盖顶层标量;像具体规则级别这样的嵌套设置没有对应的环境变量。请使用配置文件(或 `extends` 层)来调整这类设置。

### `MAGO_PHP_VERSION`

覆盖 `php-version`。可在不修改配置的情况下,针对多个 PHP 版本测试同一份代码。

```sh
MAGO_PHP_VERSION=8.2 mago lint
```

### `MAGO_THREADS`

覆盖 `threads`。

```sh
MAGO_THREADS=4 mago lint
```

### `MAGO_STACK_SIZE`

覆盖 `stack-size`,以字节为单位。超出范围的值会被限制到受支持的区间 (最小 2 MiB,最大 8 MiB)。

```sh
MAGO_STACK_SIZE=8388608 mago lint
```

### `MAGO_EDITOR_URL`

覆盖 `editor-url` 以及自动检测到的编辑器 URL。在诊断输出中可点击文件路径的所有输入里,优先级最高。受支持的模板见 [编辑器集成小节](/guide/configuration/#editor-integration)。

```sh
MAGO_EDITOR_URL="phpstorm://open?file=%file%&line=%line%&column=%column%" mago lint
```

### `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION`

覆盖 `allow-unsupported-php-version`。设为 `true` 可让 Mago 运行在它官方不支持的 PHP 版本上。不建议使用。

```sh
MAGO_ALLOW_UNSUPPORTED_PHP_VERSION=true mago lint
```

### `MAGO_NO_VERSION_CHECK`

覆盖 `no-version-check`。设为 `true` 可关闭"已安装二进制偏离 `mago.toml` 中锁定版本"的警告。无论该变量如何设置,主版本漂移仍然是致命错误:主版本锁定的全部意义在于阻止跨不兼容配置 schema 的运行。

```sh
MAGO_NO_VERSION_CHECK=true mago lint
```
