<?php

declare(strict_types=1);

final class Sentinel {}

final class Value
{
    public function __construct(public int $id) {}
}

final class Holder
{
    public function __construct(public Value|Sentinel|null $value) {}
}

final class Checker
{
    /** @phpstan-assert-if-true Sentinel|null $value */
    public static function isNullOrSentinelTrueAssertion(mixed $value): bool
    {
        return null === $value || $value instanceof Sentinel;
    }

    /** @phpstan-assert-if-true !Sentinel|null $value */
    public static function isNotNullOrSentinelTrueAssertion(mixed $value): bool
    {
        return !(null === $value || $value instanceof Sentinel);
    }

    /** @phpstan-assert-if-false !Sentinel|null $value */
    public static function isNullOrSentinelFalseAssertion(mixed $value): bool
    {
        return null === $value || $value instanceof Sentinel;
    }

    /** @phpstan-assert-if-false Sentinel|null $value */
    public static function isNotNullOrSentinelFalseAssertion(mixed $value): bool
    {
        return !(null === $value || $value instanceof Sentinel);
    }
}

final class Repro
{
    public function isNullOrSentinelTrueAssertion(Value|Sentinel|null $input): int
    {
        if (Checker::isNullOrSentinelTrueAssertion($input)) {
            exit;
        }

        return $input->id;
    }

    public function isNotNullOrSentinelTrueAssertion(Value|Sentinel|null $input): int
    {
        if (Checker::isNotNullOrSentinelTrueAssertion($input)) {
            return $input->id;
        }

        exit;
    }

    public function isNullOrSentinelFalseAssertion(Value|Sentinel|null $input): int
    {
        if (Checker::isNullOrSentinelFalseAssertion($input)) {
            exit;
        }

        return $input->id;
    }

    public function isNotNullOrSentinelFalseAssertion(Value|Sentinel|null $input): int
    {
        if (Checker::isNotNullOrSentinelFalseAssertion($input)) {
            return $input->id;
        }

        exit;
    }

    public function nullsafeArgument(?Holder $holder): int
    {
        if (Checker::isNotNullOrSentinelTrueAssertion($holder?->value)) {
            return $holder->value->id;
        }

        exit;
    }
}
