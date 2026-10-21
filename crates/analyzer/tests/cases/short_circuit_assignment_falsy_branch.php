<?php

declare(strict_types=1);

/**
 * @param false $_flag
 */
function short_circuit_takes_false(bool $_flag): void {}

/**
 * @param true $_flag
 */
function short_circuit_takes_true(bool $_flag): void {}

function short_circuit_takes_bool(bool $_flag): void {}

function short_circuit_takes_nullable_string(?string $_value): void {}

// Reduced from Symfony's `ContainerBuilder::createService()`.
function short_circuit_reassigned_after_nested_conjunction(bool $try, bool $lazy, ?object $proxy): void
{
    if (true === $try && $lazy && !$try = !$proxy || $proxy instanceof Exception) {
        return;
    }

    short_circuit_takes_bool($try);
    if ($try === false) {
        return;
    }

    short_circuit_takes_true($try);
}

function short_circuit_reassigned_in_conjunction_body(bool $flag, bool $other, bool $value): void
{
    if ($flag && $other && !($flag = $value)) {
        short_circuit_takes_false($flag);

        return;
    }

    short_circuit_takes_bool($flag);
    if ($flag === false) {
        return;
    }

    short_circuit_takes_true($flag);
}

function short_circuit_reassigned_in_disjunction(bool $flag, bool $other, bool $value): void
{
    if (!$flag || !$other || ($flag = $value)) {
        return;
    }

    short_circuit_takes_false($flag);
}

function short_circuit_reassigned_in_negated_disjunction(bool $flag, bool $other, bool $value): void
{
    if (!$flag || !$other || !($flag = $value)) {
        short_circuit_takes_bool($flag);

        return;
    }

    short_circuit_takes_true($flag);
}

function short_circuit_reassigned_nullable(?string $value, bool $other, ?string $replacement): void
{
    if ($value !== null && $other && ($value = $replacement) === null) {
        return;
    }

    short_circuit_takes_nullable_string($value);
    if ($value === null) {
        return;
    }

    echo $value;
}

function short_circuit_is_bool_after_falsy(bool|int $flag): void
{
    if (!$flag && is_bool($flag)) {
        short_circuit_takes_false($flag);
    }
}
