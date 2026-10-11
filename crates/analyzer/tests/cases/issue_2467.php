<?php

declare(strict_types=1);

namespace Issue2467;

/** @template T of object */
final class Box
{
    /** @param T $value */
    public function __construct(
        public readonly object $value,
    ) {}
}

/** @template T of object */
abstract class Grand
{
    /** @param Box<T> $box */
    public function handle(Box $box): void {}

    /** @param Box<T> $box */
    public static function handleStatic(Box $box): void {}

    /**
     * @param Box<T> $box
     * @return Box<T>
     */
    public function identity(Box $box): Box
    {
        return $box;
    }

    /**
     * @param Box<T> $box
     * @return Box<T>
     */
    public static function identityStatic(Box $box): Box
    {
        return $box;
    }
}

final class Foo {}

final class Bar {}

/**
 * @template T of object
 * @extends Grand<T>
 */
abstract class Mid extends Grand {}

/** @extends Mid<Foo> */
final class Leaf extends Mid
{
    /** @param Box<Foo> $box */
    public function handle(Box $box): void
    {
        parent::handle($box);
        parent::handleStatic($box);
        takesFooBox(parent::identity($box));
        takesFooBox(parent::identityStatic($box));
    }

    /** @param Box<Bar> $box */
    public function reject(Box $box): void
    {
        // @mago-expect analysis:invalid-argument
        parent::handle($box);
        // @mago-expect analysis:invalid-argument
        parent::handleStatic($box);
    }
}

/** @param Box<Foo> $_ */
function takesFooBox(Box $_): void {}

/**
 * @template TValue of object
 * @extends Grand<TValue>
 */
abstract class RenamedMid extends Grand {}

/** @extends RenamedMid<Foo> */
final class RenamedLeaf extends RenamedMid
{
    /** @param Box<Foo> $box */
    public function handle(Box $box): void
    {
        parent::handle($box);
        parent::handleStatic($box);
        takesFooBox(parent::identity($box));
        takesFooBox(parent::identityStatic($box));
    }
}

/**
 * @template T of object
 * @extends Mid<T>
 */
final class GenericLeaf extends Mid
{
    /** @param Box<T> $box */
    public function handle(Box $box): void
    {
        parent::handle($box);
        parent::handleStatic($box);
    }

    /**
     * @param Box<T> $box
     * @return Box<T>
     */
    public function throughParent(Box $box): Box
    {
        return parent::identity($box);
    }

    /**
     * @param Box<T> $box
     * @return Box<T>
     */
    public static function throughParentStatic(Box $box): Box
    {
        return parent::identityStatic($box);
    }
}

/**
 * @template TFirst of object
 * @template TSecond of object
 */
abstract class PairGrand
{
    /**
     * @param Box<TFirst> $first
     * @param Box<TSecond> $second
     * @return Box<TSecond>
     */
    public function handlePair(Box $first, Box $second): Box
    {
        return $second;
    }
}

/**
 * @template TLeft of object
 * @template TRight of object
 * @extends PairGrand<TRight, TLeft>
 */
abstract class PairMid extends PairGrand {}

/**
 * @template T of object
 * @extends PairMid<T, Foo>
 */
final class PairLeaf extends PairMid
{
    /**
     * @param Box<Foo> $first
     * @param Box<T> $second
     * @return Box<T>
     */
    public function throughParent(Box $first, Box $second): Box
    {
        return parent::handlePair($first, $second);
    }
}
