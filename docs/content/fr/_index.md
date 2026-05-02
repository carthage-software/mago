+++
title = "Mago"
description = "La chaîne d'outils PHP oxydée. Un analyseur statique, un linter et un formateur écrits en Rust."
nav_order = 10
nav_section = ""
+++
<section class="home-hero">

<div class="home-hero__main">

<div class="home-hero__plate"><span>Mago</span><span class="home-hero__plate-divider">/</span><span>Chaîne d'outils PHP</span><span class="home-hero__plate-divider">/</span><span>Carthage Software</span></div>

<h1 class="home-hero__title">Une chaîne d'outils PHP, <em>oxydée</em>.</h1>

<p class="home-hero__lede">Mago est un analyseur statique, un linter et un formateur pour PHP, écrits en Rust. Conçu pour les projets qui ont dépassé la patience de leur outillage.</p>

<div class="home-hero__cta">
<a class="button button--solid" href="/guide/getting-started/"><span>Commencer</span><span class="button__arrow">→</span></a>
<a class="button" href="/playground/"><span>Ouvrir le playground</span></a>
</div>

</div>

<div class="home-hero__art">
<img class="home-hero__logo" src="/assets/logo.webp" alt="Mago, un fennec coiffé d'un chapeau de magicien" width="416" height="500" loading="eager" decoding="async">
</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 01</span><h2 class="home-section__title">Trois outils, un seul binaire</h2></header>

<div class="feature-grid">

<article class="feature">
<span class="feature__num">01 / Analyser</span>
<h3 class="feature__name">Analyse statique</h3>
<p class="feature__body">Détectez les bugs, le code mort et les types impossibles avant la mise en production. Compatible avec les annotations Psalm et PHPStan ; comprend les génériques, les types conditionnels et l'affinage de flux.</p>
<div class="feature__stat"><strong>{{BENCH_ANALYZER_MAGO_TIME}}</strong> · {{BENCH_PROJECT_LOC}} LOC</div>
</article>

<article class="feature">
<span class="feature__num">02 / Linter</span>
<h3 class="feature__name">Linting opinié</h3>
<p class="feature__body">Un catalogue soigné de règles pour la justesse, la cohérence et la clarté. Correction à la sauvegarde quand c'est sûr. Discret quand il le faut.</p>
<div class="feature__stat"><strong>{{BENCH_LINTER_MAGO_TIME}}</strong> · même projet</div>
</article>

<article class="feature">
<span class="feature__num">03 / Formater</span>
<h3 class="feature__name">Formateur</h3>
<p class="feature__body">Un formateur déterministe qui produit une sortie stable et conventionnelle. Pas de roulette de configuration, pas de débat. Vous l'installez et vous passez à autre chose.</p>
<div class="feature__stat"><strong>{{BENCH_FORMATTER_MAGO_TIME}}</strong> · même projet</div>
</article>

</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 02</span><h2 class="home-section__title">Benchmarks</h2></header>

<p>Mesuré contre {{BENCH_PROJECT_LABEL}} sur la dernière version stable de chaque outil. Plus bas est meilleur ; la colonne « × » indique combien de fois le pair le plus lent l'est par rapport à Mago. Les chiffres proviennent du tableau de bord <a href="https://carthage-software.github.io/php-toolchain-benchmarks/?project=wordpress&kind=Analyzers">php-toolchain-benchmarks</a> (détails complets : moyenne, écart-type, max, mémoire), dernière mise à jour le {{BENCH_AGGREGATION_DATE}}.</p>

<table class="bench-table">
<thead>
<tr><th>Opération</th><th>Mago</th><th>Pair A</th><th>Pair B</th><th class="bench-table__factor">×</th></tr>
</thead>
<tbody>
<tr>
<td class="bench-table__op">Analyse statique</td>
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
<td class="bench-table__op">Formatage</td>
<td class="bench-table__mago">{{BENCH_FORMATTER_MAGO_TIME}}</td>
<td class="bench-table__other">{{BENCH_FORMATTER_PEER_A}}</td>
<td class="bench-table__other">{{BENCH_FORMATTER_PEER_B}}</td>
<td class="bench-table__factor">{{BENCH_FORMATTER_FACTOR}}</td>
</tr>
</tbody>
</table>

<p><a href="/benchmarks/">Lire la méthodologie complète →</a></p>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 03</span><h2 class="home-section__title">Installation</h2></header>

<div class="install">
<div class="install__head"><span><strong>[ INSTALL ]</strong></span><span>shell · macOS · Linux · WSL</span></div>
<pre class="install__body"><code>curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash</code></pre>
<div class="install__alt">Ou via <a href="/guide/installation/#composer">Composer</a>, <a href="/guide/installation/#homebrew">Homebrew</a>, <a href="/guide/installation/#cargo">Cargo</a>, ou une <a href="/recipes/docker/">image Docker préconstruite</a>.</div>
</div>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 04</span><h2 class="home-section__title">Trois étapes pour la première exécution</h2></header>

<ol class="home-steps">
<li><strong>Installer.</strong> Une seule commande. Aucun runtime PHP requis. Un seul binaire statique.</li>
<li><strong>Initialiser.</strong> Lancez <code>mago init</code> à la racine du projet. Mago détecte votre arborescence et écrit un <code>mago.toml</code>.</li>
<li><strong>Exécuter.</strong> Utilisez <code>mago analyze</code>, <code>mago lint</code> ou <code>mago fmt</code>. Branchez-le à un pre-commit, à la CI ou à votre éditeur.</li>
</ol>

</section>

<section class="home-section">

<header class="home-section__head"><span class="home-section__num">§ 05</span><h2 class="home-section__title">Sponsors</h2></header>

<p>Mago est libre et open source, construit et maintenu par <a href="https://github.com/azjezz">Seifeddine Gmati</a> avec le soutien de ces entreprises et particuliers.</p>

<div id="home-sponsors" aria-live="polite"></div>

<div class="sponsors-cta">
<p>Vous voulez soutenir le développement de Mago ?</p>
<a class="button button--solid" href="https://github.com/sponsors/azjezz" target="_blank" rel="noopener"><span>Devenir sponsor</span><span class="button__arrow">→</span></a>
</div>

</section>

<section class="home-coda">

<h2 class="home-coda__title">Essayez sans installer</h2>

<p class="home-coda__body">Le playground exécute l'analyseur Mago complet dans votre navigateur via WebAssembly. Collez n'importe quel code PHP, partagez le résultat par URL.</p>

<a class="button button--solid" href="/playground/"><span>Ouvrir le playground</span><span class="button__arrow">→</span></a>

</section>
