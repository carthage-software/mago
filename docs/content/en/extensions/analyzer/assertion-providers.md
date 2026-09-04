+++
title = "Assertion providers"
description = "Teach Mago flow-sensitive facts established by framework assertions."
nav_order = 40
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Assertion providers

Assertion providers describe facts established by a function or method call. Mago applies those facts to the actual argument expressions, allowing ordinary control-flow analysis to narrow types afterward.

Implement `FunctionAssertionProvider` or `MethodAssertionProvider` and return the corresponding function or method targets.

Place assertion providers under `src/Mago/Analyzer/Providers/`.

## Basic type guard

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Providers;

use Acme\Domain\User;
use Mago\Sdk\Analyzer\AssertionProviderContext;
use Mago\Sdk\Analyzer\Assertion\TypeAssertion;
use Mago\Sdk\Analyzer\Assertion\TypeAssertionKind;
use Mago\Sdk\Analyzer\FunctionAssertionProvider;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\InvocationAssertions;
use Mago\Sdk\Analyzer\Type;

final class IsUserAssertion implements FunctionAssertionProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('Acme\\is_user')];
    }

    public function getAssertions(AssertionProviderContext $context): ?InvocationAssertions
    {
        return new InvocationAssertions(
            ifTrueAssertions: [
                '$value' => [
                    new TypeAssertion(
                        TypeAssertionKind::IsType,
                        Type::namedObject(User::class),
                    ),
                ],
            ],
        );
    }
}
```

Map keys are callable parameter names such as `$value`, not local variable names at the call site. Mago resolves the parameter to its actual argument expression. Every mapped fact list must be non-empty.

Return `null` to delegate to the next matching provider. An empty `InvocationAssertions` contributes no facts.

## When assertions apply

`InvocationAssertions` has three maps:

| Map | Application |
| :--- | :--- |
| `assertions` | Always after the call completes normally |
| `ifTrueAssertions` | In the branch where the call result is truthy |
| `ifFalseAssertions` | In the branch where the call result is falsy |

For a throwing assertion such as `assert_string($value)`, use `assertions`. For a predicate such as `is_string($value)`, use the conditional maps.

## Assertion families

The SDK represents Mago's assertion vocabulary without flattening it into strings:

### Type assertions

`TypeAssertion` combines a `Type` with one of:

- `IsType` or `IsNotType`;
- `IsIdentical` or `IsNotIdentical`;
- `IsEqual` or `IsNotEqual`;
- `InArray` or `NotInArray`.

### Simple assertions

`SimpleAssertion` carries a `SimpleAssertionKind`:

- `Any`, `Falsy`, or `Truthy`;
- `IsEqualIsset`, `IsIsset`, or `IsNotIsset`;
- `HasStringArrayAccess` or `HasIntOrStringArrayAccess`;
- `ArrayKeyExists` or `ArrayKeyDoesNotExist`;
- `Empty`, `NonEmpty`, `EmptyCountable`, or `Countable`.

### Integer and count assertions

`IntegerAssertion` supports collection counts, integer comparisons, derived bounds, and string-length bounds. Count values must be non-negative.

### Array-key assertions

`ArrayKeyAssertion` carries an `ArrayKey` and one of:

- `HasKey`;
- `DoesNotHaveKey`;
- `HasNonnullEntryForKey`;
- `DoesNotHaveNonnullEntryForKey`.

### Countability assertions

`CountabilityAssertion` represents `NonEmpty` or `NotCountable` and retains whether Mago may safely negate the fact.

### Variable comparisons

`VariableAssertion` relates the asserted expression to another tracked variable with less-than, less-than-or-equal, greater-than, or greater-than-or-equal. Its variable identifier must be non-empty.

## Context and performance

`AssertionProviderContext` provides the same PHP version, codebase, invocation, type comparator, and cancellation services as a return-type provider. Argument types have already been inferred.

Target only the assertion helpers the plugin understands. Assertions are called in a hot analysis path; avoid codebase queries or type comparisons unless the result genuinely depends on them.
