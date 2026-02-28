---
layout: home

hero:
  name: "Mago"
  text: "90x faster PHP tooling"
  tagline: Stop waiting. Start shipping. The complete PHP toolchain that analyzes your entire codebase in seconds, not minutes.
  image:
    src: /assets/logo.png
    alt: Mago
    id: mago-logo
  actions:
    - theme: brand
      text: Install in 10 seconds
      link: /guide/installation
    - theme: alt
      text: Try the Playground
      link: /playground

features:
  - icon: 🔍
    title: Static Analyzer
    details: Catch type errors, null pointer exceptions, and logic bugs before they hit production. No more runtime surprises.
  - icon: 🎨
    title: Code Formatter
    details: Automatic PER-CS formatting. End style debates forever. One command, consistent code across your entire team.
  - icon: 🛡️
    title: Smart Linter
    details: Find code smells, security issues, and anti-patterns. 100+ rules to keep your codebase clean and maintainable.
---

<div class="quick-install">
  <h2>Get started in seconds</h2>
  <div class="install-code">

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash
```

  </div>
  <p class="install-alt">
    Or install via <a href="/guide/installation#composer-php-project">Composer</a>, <a href="/guide/installation#homebrew-macos">Homebrew</a>, or <a href="/guide/installation#cargo-rust">Cargo</a>
  </p>
</div>

<div class="speed-comparison">
  <h2>How fast is "blazing fast"?</h2>
  <p class="speed-subtitle">Benchmarked on real-world, large-scale PHP codebases</p>
  <div class="speed-grid">
    <div class="speed-card">
      <div class="speed-label">Static Analysis</div>
      <div class="speed-stats">
        <div class="speed-mago"><strong>Mago:</strong> 1.84s</div>
        <div class="speed-others">PHPStan: 81.6s · Psalm: 21.1s</div>
      </div>
      <div class="speed-badge">44x faster</div>
    </div>
    <div class="speed-card">
      <div class="speed-label">Linting</div>
      <div class="speed-stats">
        <div class="speed-mago"><strong>Mago:</strong> 0.55s</div>
        <div class="speed-others">PHP-CS-Fixer: 49.6s · Pint: 31.1s</div>
      </div>
      <div class="speed-badge">90x faster</div>
    </div>
    <div class="speed-card">
      <div class="speed-label">Formatting</div>
      <div class="speed-stats">
        <div class="speed-mago"><strong>Mago:</strong> 0.36s</div>
        <div class="speed-others">Pretty PHP: 31.4s</div>
      </div>
      <div class="speed-badge">86x faster</div>
    </div>
  </div>
  <p class="speed-footer"><a href="/benchmarks">View full benchmarks →</a></p>
</div>

<div class="how-it-works">
  <h2>Up and running in 3 steps</h2>
  <div class="steps">
    <div class="step">
      <div class="step-number">1</div>
      <div class="step-content">
        <h3>Install</h3>
        <p>One command. No PHP required. Works on macOS, Linux, and Windows.</p>
      </div>
    </div>
    <div class="step">
      <div class="step-number">2</div>
      <div class="step-content">
        <h3>Initialize</h3>
        <p>Run <code>mago init</code> in your project. Mago auto-detects your setup.</p>
      </div>
    </div>
    <div class="step">
      <div class="step-number">3</div>
      <div class="step-content">
        <h3>Run</h3>
        <p>Use <code>mago analyze</code> to analyze, <code>mago lint</code> to lint, <code>mago fmt</code> to format.</p>
      </div>
    </div>
  </div>
</div>

---

<div class="consulting-section">
  <div class="consulting-badge">From the creators of Mago</div>
  <h2>Carthage Software</h2>
  <p class="consulting-lead">
    We're a PHP & Rust consultancy that helps teams build faster, more reliable software.
  </p>
  <div class="consulting-services">
    <div class="service-item">
      <strong>Performance & Scaling ⚡</strong>
      <p>Optimize slow applications and prepare for growth</p>
    </div>
    <div class="service-item">
      <strong>PHP Modernization 🔄</strong>
      <p>Migrate legacy code to modern PHP 8.x standards</p>
    </div>
    <div class="service-item">
      <strong>Rust Development 🦀</strong>
      <p>High-performance tools and PHP extensions in Rust</p>
    </div>
    <div class="service-item">
      <strong>Code Quality ✨</strong>
      <p>Audits, reviews, and best practices implementation</p>
    </div>
  </div>
  <a href="https://carthage.software/en/#contact" class="consulting-cta" rel="noopener">
    Let's talk about your project →
  </a>
</div>

<div class="sponsors-section" id="our-sponsors">
  <div class="sponsors-header">
    <h2>Our Sponsors</h2>
    <p>Mago is free and open source, made possible by these amazing sponsors.</p>
  </div>
  <!-- SPONSORS_START -->
<div class="sponsor-tier"><div class="sponsor-tier-grid"><a href="https://www.jetbrains.com/" title="JetBrains" target="_blank" rel="noopener" class="sponsor-card sponsor-gold"><img src="https://avatars.githubusercontent.com/u/60931315?u=f9b545e50cace9e9028f77eaf1e83104d18d4d48&v=4&s=128" alt="JetBrains"><span>JetBrains</span></a>
</div></div>
<div class="sponsor-tier"><div class="sponsor-tier-grid"><a href="https://github.com/jasonrm" title="Jason R. McNeil" target="_blank" rel="noopener" class="sponsor-card sponsor-silver"><img src="https://avatars.githubusercontent.com/u/39949?u=69c0e4fb08c439250978d41dbc3371d2f0609b98&v=4&s=80" alt="Jason R. McNeil"><span>Jason R. McNeil</span></a>
<a href="https://ofcompute.rs/" title="Vincent Berset" target="_blank" rel="noopener" class="sponsor-card sponsor-silver"><img src="https://avatars.githubusercontent.com/u/5173120?u=95efc76cd8fc804536dc6dd25781a95b650bf902&v=4&s=80" alt="Vincent Berset"><span>Vincent Berset</span></a>
</div></div>
<div class="sponsor-tier"><div class="sponsor-tier-grid"><a href="https://heysora.net/" title="HeySora" target="_blank" rel="noopener" class="sponsor-card sponsor-bronze"><img src="https://avatars.githubusercontent.com/u/17962248?u=3cd88b4479a0dd018738612569d0954c81cdd25f&v=4&s=64" alt="HeySora"><span>HeySora</span></a>
<a href="https://github.com/kambo-1st" title="Bohuslav Šimek" target="_blank" rel="noopener" class="sponsor-card sponsor-bronze"><img src="https://avatars.githubusercontent.com/u/6493048?u=5eddf1eb923810745d8bdd62496d245238833d07&v=4&s=64" alt="Bohuslav Šimek"><span>Bohuslav Šimek</span></a>
<a href="https://www.ticketswap.com" title="TicketSwap" target="_blank" rel="noopener" class="sponsor-card sponsor-bronze"><img src="https://avatars.githubusercontent.com/u/5766233?v=4&s=64" alt="TicketSwap"><span>TicketSwap</span></a>
</div></div>
<div class="sponsor-tier"><div class="sponsor-tier-grid"><a href="http://julian.center" title="Julian Schramm" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/79840527?u=2e15d0ec9e6d0bf6554ef69af6b1dba32a051417&v=4&s=48" alt="Julian Schramm"><span>Julian Schramm</span></a>
<a href="https://bertell.fi" title="Rasmus Bertell" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/57398362?u=f9bb382310c9552de2a7a82193c86fb94750e971&v=4&s=48" alt="Rasmus Bertell"><span>Rasmus Bertell</span></a>
<a href="https://afilina.com" title="Anna Filina" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/199835?u=d133afc4e19961b7da45b6bb485ed9ac9232e4bb&v=4&s=48" alt="Anna Filina"><span>Anna Filina</span></a>
<a href="https://github.com/Zuruuh" title="Zuruh" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/77583853?u=568441a6191e891cf4c9f52c9c230c45930bed86&v=4&s=48" alt="Zuruh"><span>Zuruh</span></a>
<a href="http://yigit.dev" title="Yigit Cukuren" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/42731194?u=bb0fb4e3635f7fcab7ff91ff4b311fd5a99cb009&v=4&s=48" alt="Yigit Cukuren"><span>Yigit Cukuren</span></a>
<a href="https://github.com/M-arcus" title="Marcus Müller" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/25648755?u=ed5653c75b47a8da04dd642ce15a5efb09778b1e&v=4&s=48" alt="Marcus Müller"><span>Marcus Müller</span></a>
<a href="https://github.com/sdrewergutland" title="Sebastian Drewer-Gutland" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/16477894?u=fd958b1c5c1ddc9259a5dd7dfa6dadc9ace88a11&v=4&s=48" alt="Sebastian Drewer-Gutland"><span>Sebastian Drewer-Gutland</span></a>
<a href="https://innocenzi.dev" title="Enzo Innocenzi" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/16060559?u=4c0ac72d014627d9dce1fa42e530b9491dc9cfae&v=4&s=48" alt="Enzo Innocenzi"><span>Enzo Innocenzi</span></a>
<a href="https://danielweaver.dev" title="Daniel Weaver" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/3847288?u=717dd9991e86a001567e197fe3ba5d618a300118&v=4&s=48" alt="Daniel Weaver"><span>Daniel Weaver</span></a>
<a href="http://r0.fyi" title="Anatolij Vasilev" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/3026792?u=bff97cda9842e84032af0ad0e7b0008b5c7a8b7f&v=4&s=48" alt="Anatolij Vasilev"><span>Anatolij Vasilev</span></a>
<a href="https://devnix.dev/" title="Pablo Largo Mohedano" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/1777519?v=4&s=48" alt="Pablo Largo Mohedano"><span>Pablo Largo Mohedano</span></a>
<a href="https://github.com/ddanielou" title="ddanielou" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/1174337?v=4&s=48" alt="ddanielou"><span>ddanielou</span></a>
<a href="https://github.com/JacerOmri" title="Jacer OMRI" target="_blank" rel="noopener" class="sponsor-card sponsor-supporter"><img src="https://avatars.githubusercontent.com/u/537996?u=6d425f3166884ff0c0da4ffc252d6388c9940714&v=4&s=48" alt="Jacer OMRI"><span>Jacer OMRI</span></a>
</div></div>

<!-- SPONSORS_END -->
  <div class="sponsor-cta">
    <p>Want to support Mago's development?</p>
    <a href="https://github.com/sponsors/azjezz" target="_blank" rel="noopener" class="sponsor-button">Become a Sponsor</a>
  </div>
</div>

<style>
.vp-doc hr {
  max-width: 800px;
  margin: 4rem auto;
}

.quick-install {
  max-width: 700px;
  margin: 4rem auto;
  text-align: center;
}

.quick-install h2 {
  font-size: 1.8rem;
  margin-bottom: 1.5rem;
}

.install-code {
  background: var(--vp-c-bg-soft);
  border-radius: 8px;
  padding: 0;
  margin-bottom: 1rem;
}

.install-code pre {
  margin: 0 !important;
}

.install-alt {
  color: var(--vp-c-text-2);
  font-size: 0.9rem;
}

.speed-comparison {
  max-width: 900px;
  margin: 4rem auto;
  text-align: center;
}

.speed-comparison h2 {
  font-size: 1.8rem;
  margin-bottom: 0.5rem;
}

.speed-subtitle {
  color: var(--vp-c-text-2);
  margin-bottom: 2rem;
}

.speed-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 1.5rem;
}

.speed-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 12px;
  padding: 1.5rem;
  text-align: left;
}

.speed-label {
  font-weight: 600;
  font-size: 0.9rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--vp-c-text-2);
  margin-bottom: 0.75rem;
}

.speed-mago {
  font-size: 1.1rem;
  margin-bottom: 0.25rem;
}

.speed-others {
  font-size: 0.85rem;
  color: var(--vp-c-text-3);
}

.speed-badge {
  display: inline-block;
  margin-top: 1rem;
  padding: 0.25rem 0.75rem;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: 600;
}

.speed-footer {
  margin-top: 1rem;
}

.how-it-works {
  max-width: 800px;
  margin: 4rem auto;
  text-align: center;
}

.how-it-works h2 {
  font-size: 1.8rem;
  margin-bottom: 2rem;
}

.steps {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  text-align: left;
}

.step {
  display: flex;
  align-items: flex-start;
  gap: 1.25rem;
}

.step-number {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  background: var(--vp-c-brand-1);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 1.1rem;
}

.step-content h3 {
  margin: 0 0 0.25rem 0;
  font-size: 1.1rem;
}

.step-content p {
  margin: 0;
  color: var(--vp-c-text-2);
}

.step-content code {
  background: var(--vp-c-bg-soft);
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  font-size: 0.9rem;
}

.consulting-section {
  max-width: 800px;
  margin: 4rem auto;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 16px;
  padding: 2.5rem;
  text-align: center;
}

.consulting-badge {
  display: inline-block;
  padding: 0.35rem 1rem;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  font-size: 0.8rem;
  font-weight: 600;
  border-radius: 20px;
  margin-bottom: 0.5rem;
}

.consulting-section h2 {
  font-size: 1.75rem;
  margin: 1.5rem 0 1.5rem 0;
  color: var(--vp-c-text-1);
}

.consulting-lead {
  color: var(--vp-c-text-2);
  font-size: 1.1rem;
  max-width: 400px;
  margin: 1.5rem auto 2.5rem auto !important;
  text-align: center;
}

.consulting-services {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1.25rem;
  text-align: left;
  margin-bottom: 2rem;
}

.service-item strong {
  display: block;
  font-size: 0.95rem;
  margin-bottom: 0.25rem;
}

.service-item p {
  margin: 0;
  font-size: 0.85rem;
  color: var(--vp-c-text-2);
}

.consulting-cta {
  display: inline-block;
  padding: 0.875rem 2rem;
  background: var(--vp-c-brand-1);
  color: white !important;
  font-weight: 600;
  font-size: 1rem;
  text-decoration: none;
  border-radius: 8px;
  transition: all 0.2s;
}

.consulting-cta:hover {
  background: var(--vp-c-brand-2);
  transform: translateY(-2px);
}

.sponsors-section {
  max-width: 800px;
  margin: 4rem auto;
  text-align: center;
}

.sponsors-header {
  margin-bottom: 2rem;
}

.sponsors-header h2 {
  font-size: 1.5rem;
  margin-bottom: 0.5rem;
}

.sponsors-header p {
  color: var(--vp-c-text-2);
  font-size: 0.95rem;
}

.sponsor-tier {
  margin-bottom: 1.5rem;
}

.sponsor-tier-grid {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 1rem;
}

.sponsor-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem;
  border-radius: 12px;
  text-decoration: none;
  transition: all 0.2s;
  min-width: 120px;
}

.sponsor-card:hover {
  background: var(--vp-c-bg-soft);
}

.sponsor-card img {
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid var(--vp-c-divider);
  transition: border-color 0.2s;
}

.sponsor-card:hover img {
  border-color: var(--vp-c-brand-1);
}

.sponsor-card span {
  font-size: 0.8rem;
  font-weight: 500;
  color: var(--vp-c-text-2);
  max-width: 100px;
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sponsor-card:hover span {
  color: var(--vp-c-brand-1);
}

.sponsor-gold img {
  width: 72px;
  height: 72px;
}

.sponsor-silver img {
  width: 56px;
  height: 56px;
}

.sponsor-bronze img {
  width: 48px;
  height: 48px;
}

.sponsor-supporter img {
  width: 40px;
  height: 40px;
}

.sponsor-cta {
  padding: 1.5rem;
  background: var(--vp-c-bg-soft);
  border-radius: 12px;
}

.sponsor-cta p {
  margin: 0 0 1rem 0 !important;
  color: var(--vp-c-text-2);
}

.sponsor-button {
  display: inline-block;
  padding: 0.625rem 1.5rem;
  background: transparent;
  border: 2px solid var(--vp-c-brand-1);
  color: var(--vp-c-brand-1) !important;
  font-weight: 600;
  font-size: 0.9rem;
  text-decoration: none;
  border-radius: 8px;
  transition: all 0.2s;
}

.sponsor-button:hover {
  background: var(--vp-c-brand-1);
  color: white !important;
}
</style>
