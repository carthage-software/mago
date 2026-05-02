+++
title = "Mago"
description = "The oxidized PHP toolchain. A static analyzer, linter, and formatter written in Rust."
nav_order = 10
nav_section = ""
+++
<section class="home-hero">

<div class="home-hero__main">

<div class="home-hero__plate"><span>Mago</span><span class="home-hero__plate-divider">/</span><span>PHP toolchain</span><span class="home-hero__plate-divider">/</span><span>Carthage Software</span></div>

<h1 class="home-hero__title">A PHP toolchain, <em>oxidized</em>.</h1>

<p class="home-hero__lede">Mago is a static analyzer, linter, and formatter for PHP, written in Rust. Built for projects that have outgrown the patience of their tooling.</p>

<div class="home-hero__cta">
<a class="button button--solid" href="/guide/getting-started/"><span>Get started</span><span class="button__arrow">→</span></a>
<a class="button" href="/playground/"><span>Open the playground</span></a>
</div>

</div>

<div class="home-hero__art">
<img class="home-hero__logo" src="/assets/logo.webp" alt="Mago, a fennec fox wearing a wizard's hat" width="416" height="500" loading="eager" decoding="async">
</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 01</span><h2 class="home-section__title">Three tools, one binary</h2></header>

<div class="feature-grid">

<article class="feature">
<span class="feature__num">01 / Analyze</span>
<h3 class="feature__name">Static analysis</h3>
<p class="feature__body">Find bugs, dead code, and impossible types before they ship. Compatible with Psalm and PHPStan annotations; understands generics, conditional types, and flow narrowing.</p>
<div class="feature__stat"><strong>{{BENCH_ANALYZER_MAGO_TIME}}</strong> · {{BENCH_PROJECT_LOC}} LOC</div>
</article>

<article class="feature">
<span class="feature__num">02 / Lint</span>
<h3 class="feature__name">Opinionated linting</h3>
<p class="feature__body">A curated catalogue of rules for correctness, consistency, and clarity. Fix-on-save where safe. Quiet where it should be.</p>
<div class="feature__stat"><strong>{{BENCH_LINTER_MAGO_TIME}}</strong> · same project</div>
</article>

<article class="feature">
<span class="feature__num">03 / Format</span>
<h3 class="feature__name">Formatter</h3>
<p class="feature__body">A deterministic formatter that produces stable, conventional output. No configuration roulette, no debate. Drop in and move on.</p>
<div class="feature__stat"><strong>{{BENCH_FORMATTER_MAGO_TIME}}</strong> · same project</div>
</article>

</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 02</span><h2 class="home-section__title">Benchmarks</h2></header>

<p>Measured against {{BENCH_PROJECT_LABEL}} on the latest stable release of every tool. Lower is better; the "×" column shows how many times slower the slowest peer is compared to Mago. Numbers refresh from the <a href="https://carthage-software.github.io/php-toolchain-benchmarks/?project=wordpress&kind=Analyzers">php-toolchain-benchmarks</a> dashboard (full mean / stddev / max / memory breakdown there), last updated {{BENCH_AGGREGATION_DATE}}.</p>

<table class="bench-table">
<thead>
<tr><th>Operation</th><th>Mago</th><th>Peer A</th><th>Peer B</th><th class="bench-table__factor">×</th></tr>
</thead>
<tbody>
<tr>
<td class="bench-table__op">Static analysis</td>
<td class="bench-table__mago">{{BENCH_ANALYZER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_ANALYZER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_ANALYZER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_ANALYZER_FACTOR}}</td>
</tr>
<tr>
<td class="bench-table__op">Linting</td>
<td class="bench-table__mago">{{BENCH_LINTER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_LINTER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_LINTER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_LINTER_FACTOR}}</td>
</tr>
<tr>
<td class="bench-table__op">Formatting</td>
<td class="bench-table__mago">{{BENCH_FORMATTER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_FORMATTER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_FORMATTER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_FORMATTER_FACTOR}}</td>
</tr>
</tbody>
</table>

<p><a href="/benchmarks/">Read the full methodology →</a></p>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 03</span><h2 class="home-section__title">Install</h2></header>

<div class="install">
<div class="install__head"><span><strong>[ INSTALL ]</strong></span><span>shell · macOS · Linux · WSL</span></div>
<pre class="install__body"><code>curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash</code></pre>
<div class="install__alt">Or via <a href="/guide/installation/#composer">Composer</a>, <a href="/guide/installation/#homebrew">Homebrew</a>, <a href="/guide/installation/#cargo">Cargo</a>, or a <a href="/recipes/docker/">prebuilt Docker image</a>.</div>
</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 04</span><h2 class="home-section__title">Three steps to first run</h2></header>

<ol class="home-steps">
<li><strong>Install.</strong> One command. No PHP runtime required. Single static binary.</li>
<li><strong>Initialize.</strong> Run <code>mago init</code> in your project root. Mago detects your layout and writes a <code>mago.toml</code>.</li>
<li><strong>Run.</strong> Use <code>mago analyze</code>, <code>mago lint</code>, or <code>mago fmt</code>. Wire it into pre-commit, CI, or your editor.</li>
</ol>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 05</span><h2 class="home-section__title">Sponsors</h2></header>

<p>Mago is free and open source, built and maintained by <a href="https://github.com/azjezz">Seifeddine Gmati</a> with support from these companies and individuals.</p>

<div id="home-sponsors" aria-live="polite"></div>

<div class="sponsors-cta">
<p>Want to support Mago's development?</p>
<a class="button button--solid" href="https://github.com/sponsors/azjezz" target="_blank" rel="noopener"><span>Become a sponsor</span><span class="button__arrow">→</span></a>
</div>

</section>

<section class="home-coda">

<h2 class="home-coda__title">Try it without installing</h2>

<p class="home-coda__body">The playground runs the full Mago analyzer in your browser via WebAssembly. Paste any PHP, share the result by URL.</p>

<a class="button button--solid" href="/playground/"><span>Open the playground</span><span class="button__arrow">→</span></a>

</section>
