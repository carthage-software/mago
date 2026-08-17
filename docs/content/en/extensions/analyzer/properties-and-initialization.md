+++
title = "Properties and initialization"
description = "Describe framework-derived property access types and property initialization."
nav_order = 50
nav_section = "Extensions"
nav_subsection = "Analyzer"
+++
# Properties and initialization

Frameworks often expose properties dynamically or initialize declared properties outside an ordinary PHP constructor. Mago provides separate providers because these are different semantic claims.

Place these provider implementations under `src/Mago/Analyzer/Providers/`.

## Property type overrides

`PropertyTypeProvider` establishes a targeted property and supplies its read and write contracts:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Providers;

use Acme\Framework\Database\Model;
use DateTimeImmutable;
use Mago\Sdk\Analyzer\PropertyTarget;
use Mago\Sdk\Analyzer\PropertyType;
use Mago\Sdk\Analyzer\PropertyTypeProvider;
use Mago\Sdk\Analyzer\PropertyTypeProviderContext;
use Mago\Sdk\Analyzer\Type;

final class ModelProperties implements PropertyTypeProvider
{
    public function getTargets(): array
    {
        return [PropertyTarget::allProperties(Model::class)];
    }

    public function getPropertyType(PropertyTypeProviderContext $context): ?PropertyType
    {
        if ($context->access->property !== 'created_at') {
            return null;
        }

        return new PropertyType(
            readType: Type::namedObject(DateTimeImmutable::class),
            writeType: Type::union(
                Type::namedObject(DateTimeImmutable::class),
                Type::string(),
            ),
        );
    }
}
```

Property target names never include `$`. An exact target class includes descendants and implementations; terminal wildcards may target property prefixes or every property.

`PropertyAccess` contains:

- the resolved class and property name;
- `Read` or `Write` access kind;
- inferred receiver type;
- source span.

Returning `null` delegates to the next provider and native property handling. A non-null `PropertyType` must have at least one side. A `null` read type makes reads invalid; a `null` write type makes writes invalid.

Mago asks a matching provider before native property lookup, so a non-null answer can establish a dynamic property or override the access type of a declared property. Return an override only when the framework contract is authoritative. Do not use it merely to suppress an issue for a misspelled property.

## Declared property initialization

`PropertyInitializationProvider` answers whether framework behavior initializes an existing declared property:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Providers;

use Acme\Framework\DependencyInjection\Inject;
use Acme\Framework\Http\Controller;
use Mago\Sdk\Analyzer\PropertyInitializationProvider;
use Mago\Sdk\Analyzer\PropertyInitializationProviderContext;
use Mago\Sdk\Analyzer\PropertyTarget;

final class InjectedProperties implements PropertyInitializationProvider
{
    public function getTargets(): array
    {
        return [PropertyTarget::allProperties(Controller::class)];
    }

    public function isPropertyInitialized(
        PropertyInitializationProviderContext $context,
    ): bool {
        foreach ($context->property->attributes as $attribute) {
            if ($attribute->name === Inject::class) {
                return true;
            }
        }

        return false;
    }
}
```

The context contains the declaring class, full `PropertyMetadata`, codebase, type comparator, PHP version, and cancellation token. The metadata property name includes `$`; the provider target does not.

Return `true` only when initialization is guaranteed for every constructed instance represented by the target. This provider changes definite-initialization analysis; it does not change the property's declared type.

## Class initializer methods

`ClassInitializerProvider` identifies framework lifecycle methods that initialize properties:

```php
<?php

declare(strict_types=1);

namespace Acme\Mago\Analyzer\Providers;

use Acme\Framework\Component\Component;
use Mago\Sdk\Analyzer\ClassInitializerProvider;
use Mago\Sdk\Analyzer\ClassInitializerProviderContext;
use Mago\Sdk\Analyzer\ClassTarget;

final class ComponentInitializers implements ClassInitializerProvider
{
    public function getTargets(): array
    {
        return [ClassTarget::exact(Component::class)];
    }

    public function getClassInitializers(ClassInitializerProviderContext $context): array
    {
        return $context->codebase->methodExists($context->class->name, 'mount')
            ? ['mount']
            : [];
    }
}
```

Mago analyzes each returned method and its transitive calls with the same definite-initialization rules used for constructors. Every returned name must be a valid PHP method identifier. Return only lifecycle methods that the framework guarantees will run as part of instance setup.

## Choosing the right provider

| Framework behavior | Provider |
| :--- | :--- |
| Dynamic or declared property has a framework-authoritative access type | `PropertyTypeProvider` |
| Declared property is populated by dependency injection or hydration | `PropertyInitializationProvider` |
| Framework always invokes one or more methods that initialize properties | `ClassInitializerProvider` |

These providers can be combined, but each should express only its own fact. Precise separation gives Mago better diagnostics and avoids hiding genuine uninitialized-property errors.
