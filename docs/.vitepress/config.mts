import { defineConfig } from "vitepress";

const site = "https://mago.carthage.software";

export default defineConfig({
  srcDir: ".",

  title: "Mago",
  description:
    "The Oxidized PHP Toolchain: Blazing fast linter, formatter, and static analyzer for PHP, written in Rust.",
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
    ["meta", { property: "og:image", content: `${site}/assets/banner.svg` }],
    ["meta", { property: "og:url", content: site }],
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
    ["meta", { name: "twitter:image", content: `${site}/assets/banner.svg` }],
  ],
  lastUpdated: true,
  cleanUrls: true,
  themeConfig: {
    logo: "/assets/icon.svg",
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      { text: "Tools", link: "/tools/overview" },
      { text: "Benchmarks", link: "/benchmarks" },
      { text: "Sponsor", link: "https://github.com/sponsors/azjezz" },
    ],
    sidebar: [
      {
        text: "📖 Guide",
        collapsed: false,
        items: [
          { text: "Getting started", link: "/guide/getting-started" },
          { text: "Installation", link: "/guide/installation" },
          { text: "Initialization", link: "/guide/initialization" },
          { text: "Updating Mago", link: "/guide/self-update" },
        ],
      },
      {
        text: "🛠️ Tools",
        collapsed: false,
        items: [
          {
            text: "Overview",
            link: "/tools/overview",
          },
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
                collapsed: true,
                items: [
                  {
                    text: "Overview",
                    link: "/tools/linter/rules-and-categories",
                  },
                  {
                    text: "Best Practices",
                    link: "/tools/linter/rules/best-practices",
                  },
                  { text: "Clarity", link: "/tools/linter/rules/clarity" },
                  {
                    text: "Consistency",
                    link: "/tools/linter/rules/consistency",
                  },
                  {
                    text: "Correctness",
                    link: "/tools/linter/rules/correctness",
                  },
                  {
                    text: "Deprecation",
                    link: "/tools/linter/rules/deprecation",
                  },
                  {
                    text: "Maintainability",
                    link: "/tools/linter/rules/maintainability",
                  },
                  { text: "Migration", link: "/tools/linter/rules/migration" },
                  {
                    text: "Redundancy",
                    link: "/tools/linter/rules/redundancy",
                  },
                  { text: "Safety", link: "/tools/linter/rules/safety" },
                  { text: "Security", link: "/tools/linter/rules/security" },
                ],
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
              { text: "Usage", link: "/tools/analyzer/usage" },
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
        collapsed: false,
        items: [
          { text: "GitHub Actions", link: "/recipes/github-actions" },
          { text: "Zed", link: "/recipes/zed" },
          { text: "Helix", link: "/recipes/helix" },
          { text: "Visual Studio Code", link: "/recipes/vscode" },
        ],
      },
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
