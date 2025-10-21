import { defineConfig } from "vitepress";

const hostname = "https://mago.carthage.software";

export default defineConfig({
  srcDir: ".",
  title: "Mago",
  description:
    "The Oxidized PHP Toolchain: Blazing fast linter, formatter, and static analyzer for PHP, written in Rust.",
  sitemap: { hostname },
  lang: "en-US",
  head: [
    ["link", { rel: "apple-touch-icon", href: "/assets/apple-touch-icon.png" }],
    [
      "link",
      { rel: "icon", href: "/assets/favicon-32x32.png", sizes: "32x32" },
    ],
    [
      "link",
      { rel: "icon", href: "/assets/favicon-16x16.png", sizes: "16x16" },
    ],
    ["link", { rel: "icon", href: "/assets/favicon.ico" }],
    // Open Graph
    ["meta", { property: "og:type", content: "website" }],
    ["meta", { property: "og:title", content: "Mago" }],
    [
      "meta",
      {
        property: "og:description",
        content:
          "The Oxidized PHP Toolchain: Blazing fast linter, formatter, and static analyzer for PHP, written in Rust.",
      },
    ],
    [
      "meta",
      { property: "og:image", content: `${hostname}/assets/banner.svg` },
    ],
    ["meta", { property: "og:url", content: hostname }],
    ["meta", { name: "twitter:card", content: "summary_large_image" }],
    ["meta", { name: "twitter:title", content: "Mago" }],
    [
      "meta",
      {
        name: "twitter:description",
        content:
          "The Oxidized PHP Toolchain: Blazing fast linter, formatter, and static analyzer for PHP, written in Rust.",
      },
    ],
    [
      "meta",
      { name: "twitter:image", content: `${hostname}/assets/banner.svg` },
    ],
  ],
  lastUpdated: true,
  cleanUrls: true,
  themeConfig: {
    logo: "/assets/icon.svg",
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      { text: "Tools", link: "/tools/overview" },
      { text: "Benchmarks", link: "/benchmarks" },
      { text: "FAQ", link: "/faq" },
      { text: "Sponsor", link: "https://github.com/sponsors/azjezz" },
    ],
    sidebar: [
      {
        text: "📖 Guide",
        collapsed: false,
        items: [
          { text: "Getting started", link: "/guide/getting-started" },
          {
            text: "Installation",
            link: "/guide/installation",
            items: [{ text: "Upgrading", link: "/guide/upgrading" }],
          },
          { text: "Initialization", link: "/guide/initialization" },
          { text: "Configuration", link: "/guide/configuration" },
          { text: "List Files", link: "/guide/list-files" },
          {
            text: "Environment Variables",
            link: "/guide/environment-variables",
          },
        ],
      },
      {
        text: "💡 Fundamentals",
        collapsed: true,
        items: [
          {
            text: "Command-Line Interface",
            link: "/fundamentals/command-line-interface",
          },
          {
            text: "Shared Options",
            link: "/fundamentals/shared-reporting-options",
          },
          {
            text: "Suppressing Issues",
            link: "/fundamentals/suppressing-issues",
          },
          { text: "Baseline", link: "/fundamentals/baseline" },
        ],
      },
      {
        text: "🛠️ Tools",
        collapsed: true,
        items: [
          { text: "Overview", link: "/tools/overview" },
          {
            text: "Formatter",
            collapsed: true,
            items: [
              { text: "Overview", link: "/tools/formatter/overview" },
              { text: "Usage", link: "/tools/formatter/usage" },
              {
                text: "Configuration reference",
                link: "/tools/formatter/configuration-reference",
              },
              {
                text: "Command reference",
                link: "/tools/formatter/command-reference",
              },
            ],
          },
          {
            text: "Linter",
            collapsed: true,
            items: [
              { text: "Overview", link: "/tools/linter/overview" },
              { text: "Usage", link: "/tools/linter/usage" },
              {
                text: "Rules",
                link: "/tools/linter/rules-and-categories",
              },
              { text: "Integrations", link: "/tools/linter/integrations" },
              {
                text: "Configuration reference",
                link: "/tools/linter/configuration-reference",
              },
              {
                text: "Command reference",
                link: "/tools/linter/command-reference",
              },
            ],
          },
          {
            text: "Analyzer",
            collapsed: true,
            items: [
              { text: "Overview", link: "/tools/analyzer/overview" },
              {
                text: "Configuration reference",
                link: "/tools/analyzer/configuration-reference",
              },
              {
                text: "Command reference",
                link: "/tools/analyzer/command-reference",
              },
            ],
          },
          {
            text: "Architectural Guard",
            collapsed: true,
            items: [
              { text: "Overview", link: "/tools/guard/overview" },
              { text: "Usage", link: "/tools/guard/usage" },
              {
                text: "Configuration reference",
                link: "/tools/guard/configuration-reference",
              },
              {
                text: "Command reference",
                link: "/tools/guard/command-reference",
              },
            ],
          },
          {
            text: "Lexer & parser",
            collapsed: true,
            items: [
              { text: "Overview", link: "/tools/lexer-parser/overview" },
              { text: "Usage", link: "/tools/lexer-parser/usage" },
              {
                text: "Command reference",
                link: "/tools/lexer-parser/command-reference",
              },
            ],
          },
        ],
      },
      {
        text: "🧩 Recipes",
        collapsed: true,
        items: [
          { text: "GitHub Actions", link: "/recipes/github-actions" },
          { text: "Zed", link: "/recipes/zed" },
          { text: "Helix", link: "/recipes/helix" },
          { text: "Visual Studio Code", link: "/recipes/vscode" },
        ],
      },
      { text: "🤔 FAQ", link: "/faq" },
      { text: "🤝 Contributing", link: "/contributing" },
      { text: "⚡️ Benchmarks", link: "/benchmarks" },
      { text: "⭐ Projects using Mago", link: "/projects-using-mago" },
    ],
    socialLinks: [
      { icon: "github", link: "https://github.com/carthage-software/mago" },
      { icon: "twitter", link: "https://twitter.com/azjezz" },
      { icon: "discord", link: "https://discord.gg/mwyyjr27eu" },
    ],
    footer: {
      message: `Released under the MIT and/or Apache-2.0 License.<br/>Available for high-performance PHP consulting via <a href="https://carthage.software" target="_blank" rel="noopener noreferrer">carthage.software</a>.`,
      copyright: `Copyright © 2024-present <a href="https://carthage.software">carthage.software</a>`,
    },
    editLink: {
      pattern: "https://github.com/carthage-software/mago/edit/main/docs/:path",
    },
    search: {
      provider: "local",
    },
  },
});
