<?php

declare(strict_types=1);

namespace Issue2335;

final readonly class Inner {}

final readonly class Outer
{
    public function __construct(public ?Inner $inner) {}

    /** @pure */
    public function inner(): ?Inner
    {
        return $this->inner;
    }
}

final readonly class Repro
{
    public static function fromValidated(Outer $outer): Inner
    {
        self::assertValidatedMethod($outer);

        return $outer->inner();
    }

    public static function fromValidatedNotNull(Outer $outer): Inner
    {
        self::assertValidatedMethodNotNull($outer);

        return $outer->inner();
    }

    /** @phpstan-assert Inner $outer->inner() */
    private static function assertValidatedMethod(Outer $outer): void {}

    /** @phpstan-assert !null $outer->inner() */
    private static function assertValidatedMethodNotNull(Outer $outer): void {}
}

final class MutableOuter
{
    public function __construct(private ?Inner $inner) {}

    /** @pure */
    public function inner(): ?Inner
    {
        return $this->inner;
    }

    public function clear(): void
    {
        $this->inner = null;
    }
}

final class UnstableOuter
{
    public function __construct(private ?Inner $inner) {}

    public function inner(): ?Inner
    {
        return $this->inner;
    }
}

final readonly class GuardedOuter
{
    public function __construct(private ?Inner $inner) {}

    /** @pure */
    public function inner(): ?Inner
    {
        return $this->inner;
    }

    /** @phpstan-assert-if-true Inner $this->inner() */
    public function hasInner(): bool
    {
        return $this->inner !== null;
    }
}

function fromInvalidatedAssertion(MutableOuter $outer): Inner
{
    assertValidatedMutableMethod($outer);
    $outer->clear();

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $outer->inner();
}

function fromUnstableAssertion(UnstableOuter $outer): Inner
{
    assertValidatedUnstableMethod($outer);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $outer->inner();
}

function fromConditionalAssertion(GuardedOuter $outer): Inner
{
    if ($outer->hasInner()) {
        return $outer->inner();
    }

    return new Inner();
}

function afterConditionalAssertion(GuardedOuter $outer): Inner
{
    if ($outer->hasInner()) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $outer->inner();
}

/** @phpstan-assert Inner $outer->inner() */
function assertValidatedMutableMethod(MutableOuter $outer): void {}

/** @phpstan-assert Inner $outer->inner() */
function assertValidatedUnstableMethod(UnstableOuter $outer): void {}
