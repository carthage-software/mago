+++
title = "贡献指南"
description = "如何在本地搭建 Mago 开发环境、运行测试并提交修改。"
nav_order = 30
nav_section = "参考"
+++
# 为 Mago 贡献

感谢你考虑贡献。下面的步骤会带你从一份干净的仓库克隆到提交一个 pull request。

## 起步

1. 在着手任何重要的工作之前,先开一个 issue 或在已有 issue 下留言。这是确保你的工作与项目方向一致的最简单方式。

2. 在 GitHub 上 fork 仓库并克隆你的 fork:

   ```bash
   git clone https://github.com/<your-username>/mago.git
   ```

3. 安装 [Rust](https://www.rust-lang.org/tools/install) 和 [Just](https://github.com/casey/just),然后运行 `just build` 来初始化项目。Nix 用户可以先运行 `nix develop`,再执行 `just build`。

4. 创建一个分支:

   ```bash
   git checkout -b feature/my-awesome-change
   ```

5. 按照项目的代码风格进行修改。

6. 运行测试和 linter:

   ```bash
   just test
   just check
   ```

7. 提交并推送:

   ```bash
   git commit -m "feat: add my awesome change"
   git push origin feature/my-awesome-change
   ```

8. 向 [主仓库](https://github.com/carthage-software/mago) 提交 pull request。

## Pull request

bug 修复应附带一个能复现该 bug 的测试。新功能应附带全面的测试覆盖。提交贡献即表示你同意你的贡献以本项目的 MIT / Apache-2.0 双重许可证发布。

如需报告安全问题,请按照 [安全策略](https://github.com/carthage-software/mago/security/policy) 中的步骤操作。
