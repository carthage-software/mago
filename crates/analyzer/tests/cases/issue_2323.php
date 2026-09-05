<?php

declare(strict_types=1);

namespace {
    class Real {}

    class_alias(Real::class, Alias::class);
    \class_alias(Real::class, FullyQualifiedAlias::class);
    class_alias(alias: NamedAlias::class, class: Real::class);
    // @mago-format-ignore-next
    CLASS_ALIAS(Real::class, (ParenthesizedAlias::class));

    // @mago-expect analysis:non-existent-class-like
    echo UnknownClass::class;

    // @mago-expect analysis:non-existent-class-like
    class_alias(UnknownSource::class, AliasFromUnknownSource::class);

    // @mago-expect analysis:non-existent-class-like
    class_alias(alias: NamedAliasFromUnknownSource::class, class: UnknownNamedSource::class);

    /**
     * @param non-empty-string $value
     * @return non-empty-string
     */
    function identity(string $value): string
    {
        return $value;
    }

    // @mago-expect analysis:non-existent-class-like
    class_alias(Real::class, identity(UnknownNestedClass::class));

    function alias_from_missing_constant(): void
    {
        // @mago-expect analysis:non-existent-class-constant,no-value
        class_alias(Real::class, Real::MISSING);
    }
}

namespace Imported {
    use function class_alias as create_alias;

    create_alias(\Real::class, Alias::class);
    class_alias(\Real::class, FallbackAlias::class);
}

namespace Custom {
    function class_alias(string $class, string $alias): bool
    {
        return $class === $alias;
    }

    // @mago-expect analysis:non-existent-class-like
    class_alias(\Real::class, Alias::class);
}
