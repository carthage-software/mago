<?php

declare(strict_types=1);

namespace Issue2352\Concrete;

use Override;

/** @template T */
interface Equable
{
    /** @param T $other */
    public function equals(mixed $other): bool;
}

abstract readonly class Thing
{
    public function __construct(public string $value) {}
}

/** @implements Equable<Thing1> */
final readonly class Thing1 extends Thing implements Equable
{
    #[Override]
    public function equals(mixed $other): bool
    {
        return $this->value === $other->value;
    }
}

/** @implements Equable<Thing2> */
final readonly class Thing2 extends Thing implements Equable
{
    #[Override]
    public function equals(mixed $other): bool
    {
        return $this->value === $other->value;
    }
}

/** @implements Equable<NarrowWithIsA> */
final readonly class NarrowWithIsA implements Equable
{
    public function __construct(public Thing1|Thing2 $thing) {}

    #[Override]
    public function equals(mixed $other): bool
    {
        return is_a($other->thing, $this->thing::class) && $this->thing->equals($other->thing);
    }

    public function equalsInIf(self $other): bool
    {
        if (is_a($other->thing, $this->thing::class)) {
            return $this->thing->equals($other->thing);
        }

        return false;
    }
}

/** @implements Equable<NarrowWithClassConstant> */
final readonly class NarrowWithClassConstant implements Equable
{
    public function __construct(public Thing1|Thing2 $thing) {}

    #[Override]
    public function equals(mixed $other): bool
    {
        return $other->thing::class === $this->thing::class && $this->thing->equals($other->thing);
    }

    public function equalsInIf(self $other): bool
    {
        if ($other->thing::class === $this->thing::class) {
            return $this->thing->equals($other->thing);
        }

        return false;
    }
}

function invalid(Thing1|Thing2 $left, Thing1|Thing2 $right): bool
{
    // @mago-expect analysis:possibly-invalid-argument,possibly-invalid-argument
    return $left->equals($right);
}

namespace Issue2352\Generic;

use Override;

/** @template T */
interface Equable
{
    /** @param T $other */
    public function equals(mixed $other): bool;
}

/**
 * @template T of Thing
 * @implements Equable<T>
 */
abstract readonly class Thing implements Equable
{
    final public function __construct(public string $value) {}

    #[Override]
    final public function equals(mixed $other): bool
    {
        return $this->value === $other->value;
    }
}

/** @extends Thing<Thing1> */
final readonly class Thing1 extends Thing {}

/** @extends Thing<Thing2> */
final readonly class Thing2 extends Thing {}

/** @implements Equable<NarrowWithIsA> */
final readonly class NarrowWithIsA implements Equable
{
    public function __construct(public Thing1|Thing2 $thing) {}

    #[Override]
    public function equals(mixed $other): bool
    {
        return is_a($other->thing, $this->thing::class) && $this->thing->equals($other->thing);
    }
}

/** @implements Equable<NarrowWithClassConstant> */
final readonly class NarrowWithClassConstant implements Equable
{
    public function __construct(public Thing1|Thing2 $thing) {}

    #[Override]
    public function equals(mixed $other): bool
    {
        return $other->thing::class === $this->thing::class && $this->thing->equals($other->thing);
    }
}
