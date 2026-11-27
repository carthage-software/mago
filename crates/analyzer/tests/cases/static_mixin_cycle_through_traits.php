<?php

declare(strict_types=1);

// Each trait declares `@mixin Order`, so a static call Order does not declare leads back to
// Order through every one of them: the search must visit each mixin class once, or it grows
// exponentially with the traits and does not finish.

final class Query
{
    public static function whereActive(): self
    {
        return new self();
    }
}

/**
 * @mixin Order
 */
trait OrderConcern1
{
}

/**
 * @mixin Order
 */
trait OrderConcern2
{
}

/**
 * @mixin Order
 */
trait OrderConcern3
{
}

/**
 * @mixin Order
 */
trait OrderConcern4
{
}

/**
 * @mixin Order
 */
trait OrderConcern5
{
}

/**
 * @mixin Order
 */
trait OrderConcern6
{
}

/**
 * @mixin Order
 */
trait OrderConcern7
{
}

/**
 * @mixin Order
 */
trait OrderConcern8
{
}

/**
 * @mixin Order
 */
trait OrderConcern9
{
}

/**
 * @mixin Order
 */
trait OrderConcern10
{
}

/**
 * @mixin Order
 */
trait OrderConcern11
{
}

/**
 * @mixin Order
 */
trait OrderConcern12
{
}

/**
 * @mixin Query
 */
trait OrderQueries
{
}

class Order
{
    use OrderConcern1;
    use OrderConcern2;
    use OrderConcern3;
    use OrderConcern4;
    use OrderConcern5;
    use OrderConcern6;
    use OrderConcern7;
    use OrderConcern8;
    use OrderConcern9;
    use OrderConcern10;
    use OrderConcern11;
    use OrderConcern12;
    use OrderQueries;

    /**
     * @param list<mixed> $arguments
     */
    public static function __callStatic(string $name, array $arguments): mixed
    {
        // @mago-ignore analysis:string-member-selector
        return Query::$name(...$arguments);
    }
}

function find_active(): Query
{
    return Order::whereActive();
}

function find_missing(): mixed
{
    // @mago-expect analysis:non-documented-method
    return Order::missing();
}
