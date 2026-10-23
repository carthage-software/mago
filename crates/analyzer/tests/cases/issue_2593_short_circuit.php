<?php

declare(strict_types=1);

namespace ShortCircuitBranches;

function takesFalse(false $value): void {}

function takesTrue(true $value): void {}

function takesBool(bool $value): void {}

function leftAssociated(bool $flag, bool $other, bool $replacement): void
{
    if ($flag && $other && !($flag = $replacement)) {
        takesFalse($flag);

        return;
    }

    takesBool($flag);
    if ($flag === false) {
        takesFalse($flag);

        return;
    }

    takesTrue($flag);
}

function rightAssociated(bool $flag, bool $other, bool $replacement): void
{
    if ($flag && ($other && !($flag = $replacement))) {
        takesFalse($flag);

        return;
    }

    takesBool($flag);
    if ($flag === false) {
        takesFalse($flag);

        return;
    }

    takesTrue($flag);
}

function disjunction(bool $flag, bool $other, bool $replacement): void
{
    if (!$flag || !$other || ($flag = $replacement)) {
        return;
    }

    takesFalse($flag);
}

function negatedDisjunction(bool $flag, bool $other, bool $replacement): void
{
    if (!$flag || !$other || !($flag = $replacement)) {
        return;
    }

    takesTrue($flag);
}

function nullableAssignment(?string $value, bool $other, ?string $replacement): ?string
{
    if ($value !== null && $other && ($value = $replacement) === null) {
        return null;
    }

    if ($value === null) {
        return null;
    }

    return $value;
}

/**
 * @assert string $value
 * @throws \InvalidArgumentException
 */
function requireString(?string $value): bool
{
    if ($value === null) {
        throw new \InvalidArgumentException();
    }

    return rand(0, 1) === 1;
}

/** @throws \InvalidArgumentException */
function unconditionalAssertion(?string $value): string
{
    if (requireString($value) && ($value = 'fallback')) {
        return $value;
    }

    return $value;
}

/** @throws \InvalidArgumentException */
function nestedUnconditionalAssertion(?string $value, bool $other): string
{
    if ((requireString($value) && $other) && ($value = 'fallback')) {
        return $value;
    }

    return $value;
}

/** @throws \InvalidArgumentException */
function disjoinedUnconditionalAssertion(?string $value, string $replacement): string
{
    if (requireString($value) || ($value = $replacement) === '') {
        return $value;
    }

    return $value;
}

/** @throws \InvalidArgumentException */
function conditionalAssertionInTrueBranch(?string $value, bool $flag, string $replacement): string
{
    if ($flag && requireString($value) && strlen($value) > 1) {
        return $value;
    }

    return $replacement;
}

/** @throws \InvalidArgumentException */
function conditionalAssertionMayBeSkipped(?string $value, bool $flag): string
{
    if ($flag && requireString($value) && ($value = 'fallback')) {
        return $value;
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function assignedFalseInLeftBranch(bool $flag, bool $other, bool $replacement): void
{
    if (($other && !($flag = $replacement)) && falseValue($flag)) {
        takesFalse($flag);
    }
}

function falseValue(false $value): bool
{
    return rand(0, 1) === 1;
}
