<?php

declare(strict_types=1);

namespace Issue2318;

final readonly class Value
{
    public function __construct(
        public ?int $prop,
    ) {}
}

final readonly class Holder
{
    public function __construct(
        public Value $value,
    ) {}
}

final readonly class Repro
{
    public static function fromValidated(Value $input): int
    {
        self::assertValidated($input);

        return $input->prop;
    }

    public static function fromNamedArgument(Value $input): int
    {
        self::assertValidated(input: $input);

        return $input->prop;
    }

    public static function fromNestedArgument(Holder $holder): int
    {
        self::assertValidated($holder->value);

        return $holder->value->prop;
    }

    public static function fromConditionalAssertion(Value $input): int
    {
        if (self::isValidated($input)) {
            return $input->prop;
        }

        return 0;
    }

    /** @phpstan-assert !null $input->prop */
    private static function assertValidated(Value $input): void
    {
        assert($input->prop !== null);
    }

    /** @phpstan-assert-if-true !null $input->prop */
    private static function isValidated(Value $input): bool
    {
        return $input->prop !== null;
    }
}
