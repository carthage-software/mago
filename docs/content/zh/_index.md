+++
title = "Mago"
description = "氧化版 PHP 工具链。一款用 Rust 编写的静态分析器、linter 和格式化器。"
nav_order = 10
nav_section = ""
+++
<section class="home-hero">

<div class="home-hero__main">

<div class="home-hero__plate"><span>Mago</span><span class="home-hero__plate-divider">/</span><span>PHP 工具链</span><span class="home-hero__plate-divider">/</span><span>Carthage Software</span></div>

<h1 class="home-hero__title">一款<em>氧化</em>的 PHP 工具链。</h1>

<p class="home-hero__lede">Mago 是一款用 Rust 编写的 PHP 静态分析器、linter 和格式化器。专为那些现有工具链已不堪重负的项目而打造。</p>

<div class="home-hero__cta">
<a class="button button--solid" href="/guide/getting-started/"><span>快速开始</span><span class="button__arrow">→</span></a>
<a class="button" href="/playground/"><span>打开 Playground</span></a>
</div>

</div>

<div class="home-hero__art">
<img class="home-hero__logo" src="/assets/logo.webp" alt="Mago, 头戴巫师帽的耳廓狐" width="416" height="500" loading="eager" decoding="async">
</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 01</span><h2 class="home-section__title">三款工具,一个二进制</h2></header>

<div class="feature-grid">

<article class="feature">
<span class="feature__num">01 / Analyze</span>
<h3 class="feature__name">静态分析</h3>
<p class="feature__body">在代码上线前发现 bug、死代码和不可能的类型。兼容 Psalm 和 PHPStan 注解;理解泛型、条件类型和流向收窄。</p>
<div class="feature__stat"><strong>{{BENCH_ANALYZER_MAGO_TIME}}</strong> · {{BENCH_PROJECT_LOC}} 行代码</div>
</article>

<article class="feature">
<span class="feature__num">02 / Lint</span>
<h3 class="feature__name">有主张的 lint 检查</h3>
<p class="feature__body">面向正确性、一致性与清晰度的精选规则集合。安全时保存即修复。无需时保持安静。</p>
<div class="feature__stat"><strong>{{BENCH_LINTER_MAGO_TIME}}</strong> · 同一项目</div>
</article>

<article class="feature">
<span class="feature__num">03 / Format</span>
<h3 class="feature__name">格式化器</h3>
<p class="feature__body">一款确定性的格式化器,产出稳定且符合惯例的输出。无需纠结配置,没有无谓争论。开箱即用,无需多虑。</p>
<div class="feature__stat"><strong>{{BENCH_FORMATTER_MAGO_TIME}}</strong> · 同一项目</div>
</article>

</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 02</span><h2 class="home-section__title">基准测试</h2></header>

<p>对照 {{BENCH_PROJECT_LABEL}},在每个工具的最新稳定版本上测量。数值越低越好;"×"列显示最慢同类工具相对 Mago 的倍数。数据来自 <a href="https://carthage-software.github.io/php-toolchain-benchmarks/?project=wordpress&kind=Analyzers">php-toolchain-benchmarks</a> 仪表盘(包含完整的均值、标准差、最大值、内存等指标),最近一次更新:{{BENCH_AGGREGATION_DATE}}。</p>

<table class="bench-table">
<thead>
<tr><th>操作</th><th>Mago</th><th>同类 A</th><th>同类 B</th><th class="bench-table__factor">×</th></tr>
</thead>
<tbody>
<tr>
<td class="bench-table__op">静态分析</td>
<td class="bench-table__mago">{{BENCH_ANALYZER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_ANALYZER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_ANALYZER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_ANALYZER_FACTOR}}</td>
</tr>
<tr>
<td class="bench-table__op">Lint 检查</td>
<td class="bench-table__mago">{{BENCH_LINTER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_LINTER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_LINTER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_LINTER_FACTOR}}</td>
</tr>
<tr>
<td class="bench-table__op">格式化</td>
<td class="bench-table__mago">{{BENCH_FORMATTER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_FORMATTER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_FORMATTER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_FORMATTER_FACTOR}}</td>
</tr>
</tbody>
</table>

<p><a href="/benchmarks/">阅读完整方法论 →</a></p>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 03</span><h2 class="home-section__title">安装</h2></header>

<div class="install">
<div class="install__head"><span><strong>[ INSTALL ]</strong></span><span>shell · macOS · Linux · WSL</span></div>
<pre class="install__body"><code>curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash</code></pre>
<div class="install__alt">或通过 <a href="/guide/installation/#composer">Composer</a>、<a href="/guide/installation/#homebrew">Homebrew</a>、<a href="/guide/installation/#cargo">Cargo</a>,或 <a href="/recipes/docker/">预构建 Docker 镜像</a>。</div>
</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 04</span><h2 class="home-section__title">三步上手</h2></header>

<ol class="home-steps">
<li><strong>安装。</strong>一条命令。无需 PHP 运行时。单一静态二进制。</li>
<li><strong>初始化。</strong>在项目根目录运行 <code>mago init</code>。Mago 会探测你的项目布局并写入 <code>mago.toml</code>。</li>
<li><strong>运行。</strong>使用 <code>mago analyze</code>、<code>mago lint</code> 或 <code>mago fmt</code>。把它接入 pre-commit、CI 或编辑器。</li>
</ol>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 05</span><h2 class="home-section__title">赞助商</h2></header>

<p>Mago 是免费的开源项目,由 <a href="https://github.com/azjezz">Seifeddine Gmati</a> 构建并维护,得到下列公司与个人的支持。</p>

<div id="home-sponsors" aria-live="polite"></div>

<div class="sponsors-cta">
<p>想支持 Mago 的开发?</p>
<a class="button button--solid" href="https://github.com/sponsors/azjezz" target="_blank" rel="noopener"><span>成为赞助者</span><span class="button__arrow">→</span></a>
</div>

</section>

<section class="home-coda">

<h2 class="home-coda__title">无需安装即可试用</h2>

<p class="home-coda__body">Playground 通过 WebAssembly 在浏览器里运行完整的 Mago 分析器。粘贴任意 PHP 代码,通过 URL 分享结果。</p>

<a class="button button--solid" href="/playground/"><span>打开 Playground</span><span class="button__arrow">→</span></a>

</section>
