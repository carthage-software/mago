<?php

declare(strict_types=1);

abstract class Shape
{
    abstract public function area(): float;
}

/**
 * @method float area()
 */
// @mago-expect analysis:unimplemented-abstract-method
final class WithTag extends Shape {}

interface HasArea
{
    public function area(): float;
}

/**
 * @method float area()
 */
// @mago-expect analysis:unimplemented-abstract-method
final class ViaInterface implements HasArea {}

class Impl extends Shape
{
    public function area(): float
    {
        return 1.0;
    }
}

/**
 * @method float area()
 */
final class Implemented extends Impl {}
