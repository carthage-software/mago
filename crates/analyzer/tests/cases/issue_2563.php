<?php

declare(strict_types=1);

namespace Issue2563;

class Foo
{
    public bool $flag = true;

    public function isBar(): bool
    {
        return true;
    }

    public function isBaz(): bool
    {
        return true;
    }
}

final class Holder
{
    public function __construct(public ?Foo $foo) {}
}

final class Repro
{
    private const bool FALLBACK = false;

    public function falsePositive(?Foo $foo): bool
    {
        return ($foo?->isBar() ?? false) && $foo->isBaz();
    }

    public function control(?Foo $foo): bool
    {
        return $foo !== null && $foo->isBar() && $foo->isBaz();
    }

    public function property(?Foo $foo): bool
    {
        return ($foo?->flag ?? false) && $foo->isBaz();
    }

    public function guard(?Foo $foo): ?Foo
    {
        if (!($foo?->isBar() ?? false)) {
            return null;
        }

        $foo->isBaz();

        return $foo;
    }

    public function ternary(?Foo $foo): bool
    {
        return ($foo?->flag ?? false) ? $foo->isBaz() : false;
    }

    public function comparison(?Foo $foo): bool
    {
        return ($foo?->isBar() ?? false) === true && $foo->isBaz();
    }

    public function nested(?Holder $holder): bool
    {
        return ($holder?->foo?->flag ?? false) && $holder->foo->isBaz();
    }

    public function nullFallback(?Foo $foo): bool
    {
        return ($foo?->isBar() ?? null) && $foo->isBaz();
    }

    public function zeroFallback(?Foo $foo): bool
    {
        return ($foo?->flag ?? 0) && $foo->isBaz();
    }

    public function emptyStringFallback(?Foo $foo): bool
    {
        return ($foo?->flag ?? '') && $foo->isBaz();
    }

    public function zeroStringFallback(?Foo $foo): bool
    {
        return ($foo?->flag ?? '0') && $foo->isBaz();
    }

    public function emptyArrayFallback(?Foo $foo): bool
    {
        return ($foo?->flag ?? []) && $foo->isBaz();
    }

    public function constantFallback(?Foo $foo): bool
    {
        return ($foo?->flag ?? self::FALLBACK) && $foo->isBaz();
    }

    public function falsyUnionFallback(?Foo $foo, false|null $fallback): bool
    {
        return ($foo?->flag ?? $fallback) && $foo->isBaz();
    }

    public function nestedCoalesce(?Foo $foo): bool
    {
        return (($foo?->flag ?? null) ?? false) && $foo->isBaz();
    }

    public function storedGuard(?Foo $foo): bool
    {
        $hasFoo = $foo?->flag ?? false;

        return $hasFoo && $foo->isBaz();
    }

    /** @param array{foo?: Foo} $items */
    public function optionalArrayKey(array $items): bool
    {
        return ($items['foo'] ?? false) && $items['foo']->isBaz();
    }

    public function trueFallback(?Foo $foo): bool
    {
        // @mago-expect analysis:possible-method-access-on-null
        return ($foo?->isBar() ?? true) && $foo->isBaz();
    }

    public function unknownFallback(?Foo $foo, bool $fallback): bool
    {
        // @mago-expect analysis:possible-method-access-on-null
        return ($foo?->isBar() ?? $fallback) && $foo->isBaz();
    }

    public function chainedFallback(?Foo $foo, ?bool $fallback): bool
    {
        // @mago-expect analysis:possible-method-access-on-null
        return ($foo?->isBar() ?? $fallback ?? false) && $foo->isBaz();
    }

    public function strictComparisonWithZero(?Foo $foo): bool
    {
        // @mago-expect analysis:possible-method-access-on-null
        return ($foo?->flag ?? 0) !== false && $foo->isBaz();
    }

    public function strictComparisonWithNull(?Foo $foo): bool
    {
        // @mago-expect analysis:possible-method-access-on-null
        return ($foo?->flag ?? null) !== false && $foo->isBaz();
    }

    public function falseBranch(?Foo $foo): bool
    {
        // @mago-expect analysis:possible-method-access-on-null
        return ($foo?->isBar() ?? false) || $foo->isBaz();
    }

    public function negatedGuard(?Foo $foo): void
    {
        if (!($foo?->flag ?? false)) {
            // @mago-expect analysis:possible-method-access-on-null
            $foo->isBaz();
        }
    }
}
