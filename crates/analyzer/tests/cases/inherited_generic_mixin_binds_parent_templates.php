<?php

declare(strict_types=1);

/** @template TValue */
final class Box
{
    /** @return TValue */
    public function get(): mixed
    {
        exit(1);
    }
}

/**
 * @template T
 *
 * @mixin Box<T>
 */
abstract class Base
{
    /** @param list<mixed> $args */
    public function __call(string $name, array $args): mixed
    {
        return null;
    }
}

/** @extends Base<string> */
final class Child extends Base {}

/** @extends Base<int> */
abstract class Mid extends Base {}

final class Leaf extends Mid {}

/**
 * @template U
 *
 * @extends Base<U>
 */
final class Generic extends Base {}

function child_value(Child $child): string
{
    return $child->get();
}

function leaf_value(Leaf $leaf): int
{
    return $leaf->get();
}

/** @param Generic<float> $generic */
function generic_value(Generic $generic): float
{
    return $generic->get();
}
