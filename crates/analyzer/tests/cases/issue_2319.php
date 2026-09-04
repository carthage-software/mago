<?php

declare(strict_types=1);

namespace PhpstanAssertPropertyNarrowingRepro;

final readonly class Inner
{
    public function __construct(
        public ?int $prop,
    ) {}
}

final readonly class Outer
{
    public function __construct(
        public Inner $inner,
    ) {}
}

final readonly class Wrapper
{
    public function __construct(
        public Outer $outer,
        public Outer $sibling,
    ) {}

    public function fromValidated(): int
    {
        $this->assertValidated();

        return $this->outer->inner->prop;
    }

    /** @phpstan-assert !null $this->outer->inner->prop */
    private function assertValidated(): void {}
}

final readonly class Repro
{
    public static function fromValidated(Outer $outer): int
    {
        self::assertValidated($outer);

        return $outer->inner->prop;
    }

    public static function fromDeepAssertion(Wrapper $wrapper): int
    {
        self::assertDeepValidated($wrapper);

        return $wrapper->outer->inner->prop;
    }

    public static function fromNamedArgument(Outer $input): int
    {
        self::assertValidated(outer: $input);

        return $input->inner->prop;
    }

    public static function fromNestedArgument(Wrapper $wrapper): int
    {
        self::assertValidated($wrapper->outer);

        return $wrapper->outer->inner->prop;
    }

    public static function fromConditionalTrueAssertion(Outer $outer): int
    {
        if (self::isValidated($outer)) {
            return $outer->inner->prop;
        }

        return 0;
    }

    public static function fromConditionalFalseAssertion(Outer $outer): int
    {
        if (!self::isInvalid($outer)) {
            return $outer->inner->prop;
        }

        return 0;
    }

    public static function siblingRemainsNullable(Wrapper $wrapper): int
    {
        self::assertDeepValidated($wrapper);

        // @mago-expect analysis:nullable-return-statement,invalid-return-statement
        return $wrapper->sibling->inner->prop;
    }

    /** @phpstan-assert !null $outer->inner->prop */
    private static function assertValidated(Outer $outer): void {}

    /** @phpstan-assert !null $wrapper->outer->inner->prop */
    private static function assertDeepValidated(Wrapper $wrapper): void {}

    /** @phpstan-assert-if-true !null $outer->inner->prop */
    private static function isValidated(Outer $outer): bool
    {
        return $outer->inner->prop !== null;
    }

    /** @phpstan-assert-if-false !null $outer->inner->prop */
    private static function isInvalid(Outer $outer): bool
    {
        return $outer->inner->prop === null;
    }
}
