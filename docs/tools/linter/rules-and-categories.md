---
sidebar_position: 3
title: Rules & categories
---

# Rules & categories

**Mago**'s linter comes with a wide variety of rules, each designed to catch a specific type of issue.

## Rule categories

- [BestPractices](./rules/best-practices)
- [Clarity](./rules/clarity)
- [Consistency](./rules/consistency)
- [Correctness](./rules/correctness)
- [Deprecation](./rules/deprecation)
- [Maintainability](./rules/maintainability)
- [Redundancy](./rules/redundancy)
- [Safety](./rules/safety)
- [Security](./rules/security)

## Integration-specific rules


### CakePHP

- [`final-controller`](./rules/best-practices#final-controller)

### Laravel

- [`final-controller`](./rules/best-practices#final-controller)
- [`prefer-anonymous-migration`](./rules/best-practices#prefer-anonymous-migration)
- [`prefer-view-array`](./rules/best-practices#prefer-view-array)
- [`no-request-all`](./rules/safety#no-request-all)
- [`middleware-in-routes`](./rules/best-practices#middleware-in-routes)

### PHPUnit

- [`assertion-style`](./rules/consistency#assertion-style)
- [`strict-assertions`](./rules/correctness#strict-assertions)

### Psl

- [`psl-array-functions`](./rules/best-practices#psl-array-functions)
- [`psl-data-structures`](./rules/best-practices#psl-data-structures)
- [`psl-datetime`](./rules/best-practices#psl-datetime)
- [`psl-math-functions`](./rules/best-practices#psl-math-functions)
- [`psl-output`](./rules/best-practices#psl-output)
- [`psl-randomness-functions`](./rules/best-practices#psl-randomness-functions)
- [`psl-regex-functions`](./rules/best-practices#psl-regex-functions)
- [`psl-sleep-functions`](./rules/best-practices#psl-sleep-functions)
- [`psl-string-functions`](./rules/best-practices#psl-string-functions)

### Spiral

- [`final-controller`](./rules/best-practices#final-controller)

### Symfony

- [`final-controller`](./rules/best-practices#final-controller)
- [`prefer-interface`](./rules/best-practices#prefer-interface)

### Tempest

- [`final-controller`](./rules/best-practices#final-controller)

### WordPress

- [`use-wp-functions`](./rules/best-practices#use-wp-functions)
- [`no-direct-db-query`](./rules/best-practices#no-direct-db-query)
- [`no-db-schema-change`](./rules/security#no-db-schema-change)
- [`no-unescaped-output`](./rules/security#no-unescaped-output)
- [`no-roles-as-capabilities`](./rules/security#no-roles-as-capabilities)

### Yii

- [`final-controller`](./rules/best-practices#final-controller)
