+++
title = "Shell 补全"
description = "为 bash、zsh、fish、elvish 或 PowerShell 生成补全脚本。"
nav_order = 70
nav_section = "指南"
+++
# Shell 补全

`mago generate-completions <shell>` 会为你指定的 shell 打印一份补全脚本。把它保存到你的 shell 期望的位置,或者直接管道执行,这样它总是与已安装的 Mago 版本一致。

```sh
mago generate-completions fish
mago generate-completions fish | source              # fish, 临时使用
mago generate-completions zsh > ~/.zfunc/_mago      # zsh, 持久保存
mago generate-completions bash > /etc/bash_completion.d/mago
```

受支持的 shell:`bash`、`zsh`、`fish`、`elvish`、`powershell`。

## 参考

```sh
Usage: mago generate-completions <SHELL>
```

| 参数 | 说明 |
| :--- | :--- |
| `<SHELL>` | `bash`、`zsh`、`fish`、`elvish`、`powershell` 之一。 |

| 参数 | 说明 |
| :--- | :--- |
| `-h`, `--help` | 打印帮助并退出。 |
