<?php

declare(strict_types=1);

namespace Repro;

class Pivot {}

class Item {}

/**
 * @template TValue
 * @template TPivot of Pivot = Pivot
 */
class Box
{
    /**
     * @param TValue $value
     * @param TPivot $pivot
     */
    public function __construct(
        public mixed $value,
        public Pivot $pivot,
    ) {}
}

class Factory
{
    /**
     * @template TMake
     *
     * @param TMake $value
     *
     * @return Box<TMake, Pivot>
     */
    public function make(mixed $value)
    {
        return new Box($value, new Pivot());
    }
}

class Consumer
{
    /**
     * @return Box<Item>
     */
    public function broken()
    {
        return (new Factory())->make(new Item());
    }

    /**
     * @return Box<Item, Pivot>
     */
    public function explicit()
    {
        return (new Factory())->make(new Item());
    }
}

interface MethodDefaultFactory
{
    /**
     * @template T of Pivot = Pivot
     *
     * @return T
     */
    public function makeDefault(): Pivot;
}

function consume_method_default(MethodDefaultFactory $factory): Pivot
{
    return $factory->makeDefault();
}

/**
 * @template T of Pivot = \Repro\Pivot
 */
interface FullyQualifiedDefaultBox
{
    /**
     * @return T
     */
    public function get(): Pivot;
}

function consume_fully_qualified_default(FullyQualifiedDefaultBox $box): Pivot
{
    return $box->get();
}
