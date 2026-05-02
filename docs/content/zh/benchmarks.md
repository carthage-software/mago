+++
title = "基准测试"
description = "Mago 与其他 PHP 工具的对比,以及为达到这个水准所做的取舍。"
nav_order = 20
nav_section = "参考"
+++
# 基准测试

Mago 的目标是成为最快的 PHP 工具链。从语法分析器到分析器,每一个组件都围绕这个约束来设计。

我们定期在 PHP 生态的其他工具上对 Mago 进行基准测试。最新数据与历史曲线都在专门的看板上:

- [PHP Toolchain Benchmarks](https://carthage-software.github.io/php-toolchain-benchmarks/?project=psl&kind=Analyzers)
- [源码仓库](https://github.com/carthage-software/php-toolchain-benchmarks)

看板针对若干真实 PHP 代码库运行三类基准:

- 格式化器:检查整个代码库格式所需的时间。
- Linter:对整个代码库执行 lint 所需的时间。
- 分析器:在冷缓存下完整执行一次静态分析所需的时间。

## 性能承诺

速度不是目标,而是承诺。如果基准测试中列出的任何工具在同等对比下超过 Mago,我们将其视为高优先级 bug。

## 关于内存的说明

内存占用因任务而异。对于静态分析,Mago 通常比同类工具占用更少内存(在我们的运行中比 Psalm 少约 3.5 倍)。对于 lint 和格式化,Mago 可能比单线程的 PHP 工具占用更多内存。

这是有意为之。Mago 优先考虑你的时间,而不是机器资源。

为了达到我们想要的速度,Mago 使用每线程的 arena 分配器。它不会为每个小对象都向操作系统申请内存,而是预先保留大块内存,在其中以接近零成本的代价完成分配。这使得高强度并行成为可能,代价是某些工作负载下的峰值内存占用更高。我们认为,用几百兆 RAM 换回几秒(甚至几分钟)的开发者时间,是一笔划算的交易。
