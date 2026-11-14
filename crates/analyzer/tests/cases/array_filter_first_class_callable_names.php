<?php

declare(strict_types=1);

namespace Foo;

/** @assert-if-true string $value */
function is_int(mixed $value): bool
{
    return \is_string($value);
}

class Base
{
}

final class Sub extends Base
{
}

class P
{
    /** @assert-if-true Sub $b */
    public static function isSubInParent(Base $b): bool
    {
        return $b instanceof Sub;
    }
}

final class C extends P
{
    /** @assert-if-true Sub $b */
    public static function isSub(Base $b): bool
    {
        return $b instanceof Sub;
    }

    /**
     * @param list<Base> $items
     * @return array<int, Sub>
     */
    public function viaSelf(array $items): array
    {
        return array_filter($items, self::isSub(...));
    }

    /**
     * @param list<Base> $items
     * @return array<int, Sub>
     */
    public function viaStatic(array $items): array
    {
        return array_filter($items, static::isSub(...));
    }

    /**
     * @param list<Base> $items
     * @return array<int, Sub>
     */
    public function viaParent(array $items): array
    {
        return array_filter($items, parent::isSubInParent(...));
    }
}

/**
 * @param list<int|string> $values
 * @return array<int, int>
 */
function global_function(array $values): array
{
    return array_filter($values, \is_int(...));
}

/**
 * @param list<int|string> $values
 * @return array<int, string>
 */
function namespaced_function(array $values): array
{
    return array_filter($values, is_int(...));
}
