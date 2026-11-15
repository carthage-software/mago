+++
title = "Drupal 实用方案"
description = "使用官方 drupal/mago 扩展检查 Drupal 项目。"
nav_order = 70
nav_section = "实用方案"
+++
# Drupal 实用方案

对于 Drupal,请使用官方的 [`drupal/mago`](https://www.drupal.org/project/mago) 扩展。这是在 Drupal 项目中使用 Mago 的推荐方式,它补充了 Mago 本身不了解的 Drupal 知识:

- 覆盖 Mago 自带规则未涉及的 Drupal 编码标准的 linter 规则,其中许多带有修复;
- 了解 Drupal 服务、实体、配置和插件的分析器插件,并报告 Drupal 特有的错误;
- 一个为 Drupal 设置好格式化器、linter 和分析器的配置预设。

配合 Mago 自身的工具,它可以取代 phpcs、Coder、PHPStan 和 phpstan-drupal。

该扩展托管在 drupal.org 上,并有自己的[文档](https://project.pages.drupalcode.org/mago/)。以其安装指南为准,下面是简要步骤。

## 安装

```sh
composer require --dev carthage-software/mago "drupal/mago:^1.0@alpha"
```

`drupal/mago` 还没有稳定版本,所以目前 Composer 需要 `@alpha` 标记。

## 配置

在 `composer.json` 旁边创建 `mago.toml`,并继承扩展的预设:

```toml
extends = "vendor/drupal/mago/drupal.mago.toml"
php-version = "8.3"

[source]
paths = ["web/modules/custom", "web/themes/custom"]
includes = ["vendor", "web/core", "web/modules/contrib"]
```

在模块、主题或安装配置的仓库中,设置 `paths = ["."]`。在 Drupal core 的检出目录中,请改为继承 `vendor/drupal/mago/drupal-core.mago.toml`。

## 运行

使用 Composer 安装的 Mago 可执行文件,使其与扩展的协议版本一致:

```sh
vendor/bin/mago lint
vendor/bin/mago analyze
```

## 内置的 Drupal 设置

Mago 还有一个 `drupal` [linter 集成](/tools/linter/integrations/)和一个 `drupal` [格式化器预设](/tools/formatter/configuration-reference/)。这两者你都不需要设置:扩展的预设会为你选好格式化器预设,并用自己的规则取代 linter 集成所做的两处规则例外。

请在[扩展的问题队列](https://www.drupal.org/project/issues/mago)中报告 Drupal 特有的问题和需求。
