+++
title = "Extensions"
description = "Extend Mago with external linter rules and analyzer plugins."
nav_order = 10
nav_section = "Extensions"
+++
# Extensions

Mago extensions add project- and framework-specific behavior without rebuilding Mago. A configured extension host is an external program connected through Mago's language-neutral worker protocol. Each host worker advertises one or more logical extensions, and the host can be written in any language capable of implementing that protocol.

Mago bundles a first-party PHP SDK with `carthage-software/mago`. The examples and API reference in this manual use that SDK, but the host architecture does not require PHP.

An extension may contribute:

- **Linter rules** that inspect selected concrete-syntax-tree nodes and report issues or suggested edits.
- **Analyzer plugins** that provide types, callable signatures, flow assertions, framework entry points, initialization knowledge, codebase scans, and analysis hooks.
- **Worker reduction** that merges process-local data after Mago finishes using a worker pool.

The formatter and guard do not currently expose extension APIs.

## When to write an extension

An extension is appropriate when knowledge belongs to a framework, library, or application rather than PHP itself. Typical examples include:

- inferring the return type of a service-container lookup;
- teaching Mago that framework lifecycle methods initialize properties;
- marking controller actions or test methods as externally referenced;
- reporting a project-specific syntax convention;
- inspecting the final project analysis to calculate type-coverage metrics.

Fix generally applicable PHP behavior in Mago itself. Use an extension for behavior that depends on conventions or runtime wiring Mago cannot infer from PHP source alone.

## Execution and trust

Extension workers are external processes, not sandboxed scripts. They run with the operating-system permissions, working directory, and environment configured for their host. Install and enable only extensions you trust.

Mago communicates with workers through a framed binary protocol. That protocol is the extension ABI: another language can provide its own SDK or worker implementation as long as it follows the same registration, capability, framing, cancellation, and lifecycle contracts.

PHP extension authors should use the bundled SDK, which handles registration, source snapshots, analyzer metadata requests, cancellation, and responses.

One configured extension host owns a pool of identical processes. Each worker may expose one or more logical extensions. Mago uses multiple processes for CPU parallelism. The PHP SDK represents these concepts with `Worker` and `Extension`, and uses Revolt to interleave requests that cooperatively suspend.

## Where to begin

- [Getting started](/extensions/getting-started/) builds a complete linter extension.
- [Architecture and execution model](/extensions/architecture/) explains worker pools and lifecycle ordering.
- [Linter extensions](/extensions/linter/overview/) covers syntax-driven rules.
- [Analyzer plugins](/extensions/analyzer/overview/) maps the analyzer extension surface.
- [PHP SDK API index](/extensions/reference/sdk-index/) lists every public SDK type by namespace.
