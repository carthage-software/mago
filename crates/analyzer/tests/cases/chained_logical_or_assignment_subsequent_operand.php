<?php

declare(strict_types=1);

namespace Repro\ChainedLogicalOrAssignmentSubsequentOperand;

class Service
{
    public function execute(): bool
    {
        return true;
    }
}

function get_service(): ?Service
{
    return rand(0, 1) ? new Service() : null;
}

function run(mixed $value): bool
{
    return !is_string($value) || ($value = get_service()) === null || $value->execute();
}

function test_conditional_assignment(bool $flag): bool
{
    $value = null;
    // @mago-expect analysis:method-access-on-null
    return $flag && ($value = new Service()) instanceof Service || $value->execute();
}

function test_conditional_assignment_and(bool $flag): bool
{
    $value = null;
    // @mago-expect analysis:method-access-on-null
    return $flag && ($value = get_service()) !== null || $value->execute();
}

function test_unconditional_assignment_in_or(): bool
{
    return ($value = get_service()) === null || $value->execute();
}

function test_conditional_string(string $value, bool $flag): bool
{
    // @mago-expect analysis:invalid-method-access
    return $flag && ($value = new Service()) instanceof Service || $value->execute();
}

/** @param 'ab' $_value */
function accepts_ab(string $_value): bool
{
    return rand(0, 1) === 1;
}

function test_self_dependent_assignment(string $other): bool
{
    $value = 'a';

    return ($value = $value . 'b') === $other || accepts_ab($value);
}

function test_compound_assignment(string $other): bool
{
    $value = 'a';

    return ($value .= 'b') === $other || accepts_ab($value);
}

/** @param 'abc' $_value */
function accepts_abc(string $_value): bool
{
    return rand(0, 1) === 1;
}

function test_chained_compound_assignments(string $other): bool
{
    $value = 'a';

    return ($value .= 'b') === $other || ($value .= 'c') === $other || accepts_abc($value);
}

function test_conditionally_nullable_assignment(bool $flag): bool
{
    $value = null;
    // @mago-expect analysis:possible-method-access-on-null
    return $flag && ($value = get_service()) === null || $value->execute();
}

function test_right_associated_assignment(mixed $value): bool
{
    return !is_string($value) || (($value = get_service()) === null || $value->execute());
}

function test_low_precedence_or(mixed $value): bool
{
    return !is_string($value) or ($value = get_service()) === null or $value->execute();
}

function test_assignment_in_if(string $other): void
{
    $value = 'a';

    if (($value .= 'b') === $other || accepts_ab($value)) {
        accepts_ab($value);
    }

    accepts_ab($value);
}

function test_assignment_in_ternary(string $other): bool
{
    $value = 'a';

    return ($value .= 'b') === $other || accepts_ab($value) ? accepts_ab($value) : accepts_ab($value);
}

function test_reassignment_in_if(mixed $value): void
{
    if (!is_string($value) || ($value = get_service()) === null || $value->execute()) {
        return;
    }

    $value->execute();
}

function test_assignment_in_nested_right_operand(string $other): bool
{
    $value = 'a';

    return (($value .= 'b') === $other || (($value .= 'c') === $other || accepts_abc($value)))
        || accepts_abc($value);
}
