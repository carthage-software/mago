+++
title = "Linter 集成"
description = "可按项目启用的框架专属规则集。"
nav_order = 30
nav_section = "工具"
nav_subsection = "Linter"
+++
# 集成

Mago 内置了针对主流 PHP 框架、库和测试工具的规则集。当某项集成被启用时,Mago 会激活与之关联的规则。各条规则仍可在 `[linter.rules]` 中单独调整或禁用。

## 可用集成

### 框架

- CakePHP
- Laminas
- Laravel
- Neutomic
- Spiral
- Symfony
- Tempest
- Yii

### 库

- Amphp
- Carbon
- Guzzle
- Monolog
- PSL (PHP Standard Library)
- ReactPHP

### 测试

- Behat
- Codeception
- Pest
- PHPSpec
- PHPUnit

### CMS

- Drupal
- Magento
- WordPress

### ORM

- Cycle
- Doctrine

> 其中部分集成是为未来的规则预留的占位。当前已有规则的集成列表见[规则页面](/tools/linter/rules/#integration-specific-rules)。

## 启用集成

在 `mago.toml` 中,把要启用的集成列在 `[linter].integrations` 下:

```toml
[linter]
integrations = ["symfony", "phpunit"]
```

完整的配置项见[配置参考](/tools/linter/configuration-reference/)。
