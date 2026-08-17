+++
title = "Entry points and issue filtering"
description = "Declare framework-invoked methods and narrowly suppress unavoidable native diagnostics."
nav_order = 100
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Entry points and issue filtering

Entry-point declarations tell Mago that a framework invokes methods which have no direct PHP caller. Issue filters are a last resort for native diagnostics that cannot be prevented by providing better semantic information.

## Method entry points

Register a `MethodTarget` for framework-owned call paths:

```php
use Acme\Framework\Http\Controller;
use Mago\Sdk\Analyzer\MethodTarget;

$registry->registerEntryPoint(
    MethodTarget::allMethods(Controller::class),
);
```

Mago resolves matching method declarations and records native references without invoking PHP during file analysis. Exact target classes include subclasses and interface implementations.

Use the narrowest method pattern available. If only `handle` is framework-invoked, do not register every method:

```php
use Acme\Framework\Console\Command;
use Mago\Sdk\Analyzer\MethodTarget;

$registry->registerEntryPoint(
    MethodTarget::exact(Command::class, 'handle'),
);
```

## Attributed entry points

When an attribute marks invoked methods, register it directly:

```php
use Acme\Framework\Http\Controller;
use Acme\Framework\Routing\Route;
use Mago\Sdk\Analyzer\ClassTarget;

$registry->registerAttributedEntryPoint(
    ClassTarget::exact(Controller::class),
    Route::class,
);
```

The class target includes descendants and implementations. Passing a class string is shorthand for `ClassTarget::exact()`. Attribute matching and reference insertion happen natively.

Prefer entry-point registration over scanning metadata and manually adding one reference for every matching method. Manual references remain useful when route tables or container configuration determine a non-attribute relationship.

## Issue filters

`IssueFilterHook` may remove selected native analyzer issues:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Hooks;

use Mago\Sdk\Analyzer\IssueFilterContext;
use Mago\Sdk\Analyzer\IssueFilterDecision;
use Mago\Sdk\Analyzer\IssueFilterHook;

final class GeneratedProxyIssueFilter implements IssueFilterHook
{
    public function getCodes(): array
    {
        return ['non-existent-method'];
    }

    public function filterIssue(IssueFilterContext $context): IssueFilterDecision
    {
        if (!$this->isGeneratedProxyLocation($context->file, $context->issue)) {
            return IssueFilterDecision::Keep;
        }

        return IssueFilterDecision::Remove;
    }
}
```

`getCodes()` must return a non-empty list of native analyzer issue codes. Mago sends only those codes to the extension and batches all relevant issues for a file in one worker request. The PHP method remains per issue, but it does not create one IPC round trip per issue.

`IssueFilterContext` contains:

- PHP version, codebase, type comparator, and cancellation token;
- the logical file path;
- exact in-memory file contents analyzed by Mago;
- the immutable `ReportedIssue`, including level, code, message, notes, help, link, annotations, and edits.

Return `Keep` or `Remove`. Filters cannot rewrite a native issue. If the plugin needs to replace it, remove it and report a separate extension issue from an appropriate lifecycle hook.

Issue filtering is not an optional semantic hint. A filter transport, callback, or response failure propagates as an analysis-operation error; Mago does not guess whether the affected issues should be removed.

## Filters are the last resort

Before filtering an issue, check whether one of these expresses the actual semantics:

- return type or callable signature provider;
- assertion provider;
- property type provider;
- property initialization or class initializer provider;
- method entry point or symbol reference;
- initialization stub.

Semantic providers improve downstream inference. A filter merely hides one symptom and may leave other false positives or imprecise types behind.

Use source contents and annotations to make removal conditions precise. Never remove every issue of a broad code simply because a framework sometimes makes it invalid.
