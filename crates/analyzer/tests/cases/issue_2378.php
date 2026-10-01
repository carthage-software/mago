<?php

declare(strict_types=1);

namespace Issue2378;

final readonly class Foo {}

/**
 * @template TFirst of object
 * @template TSecond of object
 */
class PairParent
{
    /** @return TFirst|null */
    public function getFirst(): ?object
    {
        return null;
    }

    /** @return TSecond|null */
    public function getSecond(): ?object
    {
        return null;
    }
}

/**
 * @template T of object
 */
class GrandParentClass
{
    /** @return T|null */
    public function getFromGrandParent(): ?object
    {
        return null;
    }
}

/**
 * @template T of object
 *
 * @extends GrandParentClass<T>
 */
class MiddleClass extends GrandParentClass {}

/**
 * @template T of object
 *
 * @extends MiddleClass<T>
 */
final class LeafClass extends MiddleClass
{
    /** @return T|null */
    public function getThroughParent(): ?object
    {
        return parent::getFromGrandParent();
    }
}

/**
 * @template T of object
 *
 * @extends PairParent<Foo, T>
 */
final class PairChild extends PairParent
{
    /** @return Foo|null */
    public function getFirstFromParent(): ?Foo
    {
        return parent::getFirst();
    }

    /** @return T|null */
    public function getSecondFromParent(): ?object
    {
        return parent::getSecond();
    }
}

/**
 * @template T of object
 */
class ParentClass
{
    /** @phpstan-return T|null */
    public function getSomething(mixed $arg): ?object
    {
        return null;
    }
}

/**
 * @template T of object
 *
 * @extends ParentClass<T>
 */
final class TemplatedChildClass extends ParentClass
{
    public function getSomething(mixed $arg): ?object
    {
        return parent::getSomething($arg);
    }

    /** @phpstan-return T|null */
    public function getSomethingFromChild(mixed $arg): ?object
    {
        return parent::getSomething($arg);
    }

    /** @return T|null */
    public function getSomethingFromSelf(mixed $arg): ?object
    {
        return self::getSomething($arg);
    }

    /** @return T|null */
    public function getSomethingFromStatic(mixed $arg): ?object
    {
        return static::getSomething($arg);
    }
}

/** @extends ParentClass<Foo> */
final class FooChildClass extends ParentClass
{
    public function getSomething(mixed $arg): ?object
    {
        return parent::getSomething($arg);
    }
}

/** @param TemplatedChildClass<Foo> $child */
function useTemplatedChild(TemplatedChildClass $child): void
{
    takesNullableFoo($child->getSomething(null));
    takesNullableFoo($child->getSomethingFromChild(null));
    takesNullableFoo($child->getSomethingFromSelf(null));
    takesNullableFoo($child->getSomethingFromStatic(null));
}

function takesNullableFoo(?Foo $_): void {}
