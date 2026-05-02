+++
title = "安装"
description = "通过 shell 安装脚本、手动下载、Docker 或你所用语言的包管理器安装 Mago。"
nav_order = 20
nav_section = "指南"
+++
# 安装

Mago 以单一静态二进制的形式发布。挑一种适合你环境的安装方式即可。

## Shell 安装脚本(macOS、Linux)

macOS 和 Linux 上的推荐方式。脚本会探测你的平台、获取匹配的发行归档,并把二进制放到你的 PATH 上。

使用 `curl`:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash
```

使用 `wget`:

```sh
wget -qO- https://carthage.software/mago.sh | bash
```

### 锁定特定版本

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --version=1.25.1
```

`wget` 也支持同样的语法。

### 校验下载

如果 [GitHub CLI](https://cli.github.com/) 在你的 PATH 上,安装脚本会在解压前根据 Mago 的 GitHub 构建证明(build attestation)校验归档。无需任何参数。如果 `gh` 缺失或版本太旧,脚本会打印提示并继续运行而不进行校验。

要让校验成为强制项,请传 `--always-verify`。如果 `gh` 不可用、版本太旧,或证明不匹配,安装脚本会在动到 PATH 之前中止。

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --always-verify
```

要完全跳过校验,请传 `--no-verify`。这两个参数互斥。

## 手动下载

Windows 上的推荐方式,也是任何没有 `bash` 的系统上的不错备选。

1. 打开 [发布页面](https://github.com/carthage-software/mago/releases)。
2. 下载对应你操作系统的归档。命名遵循 `mago-<version>-<target>.tar.gz`(Windows 上是 `.zip`)。
3. 解压归档,把二进制放到 PATH 上的某个位置。

如果你保留了归档文件,可以在解压前自行校验。

```sh
VERSION=1.25.1
TARGET=x86_64-unknown-linux-gnu  # adjust for your platform
ASSET=mago-${VERSION}-${TARGET}.tar.gz

gh release download "$VERSION" --repo carthage-software/mago --pattern "$ASSET"
gh attestation verify "$ASSET" \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml

tar -xzf "$ASSET"
sudo mv "mago-${VERSION}-${TARGET}/mago" /usr/local/bin/
```

校验成功时会打印 `Verification succeeded!` 以及生成该归档的工作流运行记录。

证明绑定到归档,而非解压后的二进制。如果你只保留了二进制,就无法直接校验。请重新下载归档,完成校验,再用其中的二进制的 `sha256sum` 与系统上已有的二进制做对比。

## Docker

官方镜像基于 `scratch` 构建,大约 26 MB,可在任何运行 Docker 的地方使用,支持 `linux/amd64` 与 `linux/arm64`,无需宿主机的 PHP 运行时。

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

可用的 tag 包括 `latest`、确切版本号,以及逐级放宽的版本前缀(例如 `1.25.1`、`1.25`、`1`)。[Docker 实用方案](/recipes/docker/) 给出了 CI 示例和需要注意的限制。

## 包管理器

这些方式很方便,但依赖外部的发布节奏,通常会落后于 GitHub 发布。通过其中任何一种安装后,可运行 [`mago self-update`](/guide/upgrading/) 获取最新的官方二进制。

### Composer

适用于 PHP 项目:

```sh
composer require --dev "carthage-software/mago:^1.25.1"
```

Composer 包是一个轻量封装。第一次调用 `vendor/bin/mago` 会从 GitHub 发布下载对应的预构建二进制并缓存。后续调用复用缓存,不再发起任何网络请求。

如果 GitHub 的匿名速率限制阻止了首次下载(在共享 CI runner 上很常见),为该次调用设置 `GITHUB_TOKEN` 或 `GH_TOKEN` 即可。在 GitHub Actions 中 token 不会自动导出,需要显式传入:

```yaml
- run: vendor/bin/mago lint
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Homebrew

社区维护的 formula 通常会落后于官方发布。安装后请立即运行 `mago self-update`。

```sh
brew install mago
mago self-update
```

### Cargo

Crates.io 的发布可能比正式发布晚几个小时。和 Homebrew 一样的做法。

```sh
cargo install mago
mago self-update
```

## 校验发布的细节

每一份发行归档(各平台的 tarball、源码 tarball、源码 zip,以及 WASM 包)都在构建时通过 [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance) 进行签名。签名是一份存储在 GitHub 上的 [in-toto](https://in-toto.io/) 证明,绑定到生成该工件的工作流运行,因此校验通过的下载,可证明与 Mago 发布流水线产出的字节完全一致。

shell 安装脚本会根据你传入的参数,在三种模式之间选择。

| 模式 | 参数 | 行为 |
| :--- | :--- | :--- |
| `auto` | 无 | 如果 `gh` 可用就校验;否则跳过校验直接安装。 |
| `always` | `--always-verify` | 强制校验。`gh` 缺失、版本太旧或匹配失败都会中止安装。 |
| `never` | `--no-verify` | 即便 `gh` 可用也跳过校验。 |

校验执行的是:

```sh
gh attestation verify <archive> \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml
```

`--signer-workflow` 这一项很关键。它把证明绑定到具体的发布工作流文件。即便有人通过泄露的 GitHub Actions token 在同一仓库内触发了另一个工作流,校验也会失败。

如果校验失败,脚本会把未经校验的归档复制到当前工作目录,命名为 `<file>.unverified.tar.gz`(这样它能在临时目录清理后留存,便于你做事后取证),打印一条红色错误信息,然后在解压之前退出。任何东西都不会进入你的 PATH。

校验调用读取的是公开的证明 API,因此无需 `gh auth`。你只需要一个包含 `gh attestation` 子命令的较新版本 `gh`。

### 锁定安装脚本

`https://carthage.software/mago.sh` 会重定向到 `main` 分支上的 [`scripts/install.sh`](https://github.com/carthage-software/mago/blob/main/scripts/install.sh)。未来的修订会被自动采用,这很方便,但也意味着将来对安装脚本的改动会在没有预警的情况下生效。

为了更严格的供应链卫生,可以把脚本锁定到你审阅过的某个提交:

```sh
COMMIT=cd4cf4dfdbc72bd028ad26d11bcc815a49e27e9a  # replace with a commit you have read
curl --proto '=https' --tlsv1.2 -sSf \
  "https://raw.githubusercontent.com/carthage-software/mago/${COMMIT}/scripts/install.sh" \
  | bash -s -- --always-verify
```

GitHub 不会重写某一 SHA 上的文件,所以你审阅过的字节就是你运行的字节。更新这个锁定值是一个有意识的动作:先读懂新提交,再调整 `COMMIT`。

## 校验安装

```sh
mago --version
```

如果它打印出版本号,你就可以 [开始用 Mago 处理代码](/guide/getting-started/) 了。
