# FAQ

### Why the name "Mago"?

The project was initially named "fennec", after the fennec fox native to North Africa. However, due to a name conflict with another tool, we had to choose a new name.

We decided on "Mago" to stay true to our roots at Carthage Software. Mago of Carthage was an ancient Carthaginian writer known as the "Father of Agriculture." Just as he cultivated the land, we aim to help developers cultivate their codebases.

The name also has a wonderful double meaning. In Spanish and Italian, "mago" means "magician" or "wizard."

This blend of inspirations is perfectly captured in our logo: a fennec fox (our original mascot) dressed as a wizard, with his hat and robe adorned with the ancient Carthaginian symbol of Tanit.

### How do you pronounce Mago?

`/ˈmɑːɡoʊ/` — pronounced **mah-go**.

This breaks down to two syllables:

- **ma** as in "mama"
- **go** as in "go"

### Will Mago implement an LSP?

Yes. A Language Server Protocol (LSP) implementation is in progress. It is planned for the `1.0.0-rc.1` release and will be stabilized for the official `1.0.0` version.

### Will Mago offer editor extensions (VS Code, etc.)?

No. We will focus exclusively on implementing the LSP standard. We will not maintain editor-specific plugins or extensions. If your editor supports LSP integration (e.g., Helix, Neovim via lspconfig, VS Code with a generic client), it will work with Mago. We encourage the community to build and maintain editor-specific extensions, and we will happily feature well-regarded ones on our website.

### Will Mago support analyzer plugins?

Yes, but this is not a priority for the `1.0.0` release. Our goal is for plugins to be written in Rust, compiled to WASM, and loaded by Mago. This is a post-`1.0.0` roadmap item.

### What other PHP tools does Mago plan to replace?

Our long-term vision is for Mago to be a complete QA and development utility for PHP. While the formatter, linter, and analyzer are the core features for `1.0.0`, we plan to add more tools in the future, such as:

- A PHP version manager.
- A PHP extension installer.
- A migration tool to assist with upgrading PHP versions, frameworks, or libraries.

### Will Mago implement a Composer alternative?

No. Composer is a fantastic tool that is primarily I/O-bound. A rewrite in Rust would not yield significant performance benefits, would fragment the PHP ecosystem, and would make it very difficult to support Composer's PHP-based plugin architecture.

### Will Mago implement a PHP runtime?

Absolutely not. The PHP runtime is incredibly complex. Major efforts by large companies (e.g., Facebook's HHVM, VK's KPHP) have struggled to reach full parity with Zend Engine. Achieving this as a smaller project is infeasible and would lead to community fragmentation. We are focused on tooling, not runtimes.
