<?php

declare(strict_types=1);

function maybe_string(): ?string
{
    return rand(0, 1) ? 'hello' : null;
}

function test_or(): string
{
    $foo = maybe_string();
    $foo !== null || ($foo = '');

    return $foo;
}

function test_low_or(): string
{
    $foo = maybe_string();
    $foo !== null or ($foo = '');

    return $foo;
}

function test_and(): string
{
    $bar = maybe_string();
    $bar === null && ($bar = 'default');

    return $bar;
}

function test_low_and(): string
{
    $bar = maybe_string();
    $bar === null and ($bar = 'default');

    return $bar;
}

function takes_string(string $s): string
{
    return $s;
}

/**
 * @mago-expect analysis:possibly-null-argument
 */
function test_and_keeps_unassigned_variable(): string
{
    $nullable = maybe_string();
    $other = 'constant';
    $nullable === null && ($other = 'assigned');

    return takes_string($nullable);
}

function consume(bool $b): bool
{
    return $b;
}

/**
 * @assert string $value
 * @throws InvalidArgumentException
 */
function require_string(?string $value): bool
{
    if ($value === null) {
        throw new InvalidArgumentException();
    }

    return rand(0, 1) === 1;
}

/**
 * @throws InvalidArgumentException
 */
function test_and_preserves_unconditional_assertion(?string $value, bool $flag): string
{
    require_string($value) && consume($flag);

    return $value;
}

/**
 * @throws InvalidArgumentException
 */
function test_or_preserves_unconditional_assertion(?string $value, bool $flag): string
{
    require_string($value) || consume($flag);

    return $value;
}

function test_or_nested_left_assignment(?string $value, bool $flag): string
{
    $value !== null || ($value = 'default') && consume($flag);

    return $value;
}

function test_and_nested_left_assignment(?string $value, bool $flag): string
{
    $value === null && (($value = 'default') && consume($flag));

    return $value;
}

function test_or_keeps_unassigned_variable(bool $flag): string
{
    $value = maybe_string();
    $value !== null || consume($flag);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_and_keeps_prior_assignment(bool $flag): string
{
    $value = maybe_string();
    $value === null && consume($flag);

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_or_keeps_conditional_assignment(?string $value, bool $flag): string
{
    $value !== null || $flag && ($value = 'default');

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_and_keeps_conditional_assignment(?string $value, bool $flag): string
{
    $value === null && ($flag && ($value = 'default'));

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_or_in_if(?string $value): string
{
    if ($value !== null || ($value = '')) {
        // @mago-expect analysis:type-confirmation
        \Mago\confirm($value, 'string');
    } else {
        // @mago-expect analysis:type-confirmation
        \Mago\confirm($value, "string('')");
    }

    return $value;
}

function test_and_in_if(?string $value): string
{
    if ($value === null && ($value = 'default')) {
        // @mago-expect analysis:type-confirmation
        \Mago\confirm($value, "string('default')");
    } else {
        // @mago-expect analysis:type-confirmation
        \Mago\confirm($value, 'string');
    }

    return $value;
}

function test_or_in_if_keeps_conditional_assignment(?string $value, bool $flag): string
{
    if ($value !== null || $flag && ($value = 'default')) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_and_in_if_keeps_conditional_assignment(?string $value, bool $flag): string
{
    if ($value === null && ($flag && ($value = 'default'))) {
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_low_or_in_if(?string $value): string
{
    if ($value !== null or ($value = '')) {
        takes_string($value);
    } else {
        takes_string($value);
    }

    return $value;
}

function test_low_and_in_if(?string $value): string
{
    if ($value === null and ($value = 'default')) {
        takes_string($value);
    } else {
        takes_string($value);
    }

    return $value;
}

function test_negated_or_in_if(?string $value): string
{
    if (!($value !== null || ($value = ''))) {
        // @mago-expect analysis:type-confirmation
        \Mago\confirm($value, "string('')");
    } else {
        takes_string($value);
    }

    return $value;
}

function test_negated_and_in_if(?string $value): string
{
    if (!($value === null && ($value = 'default'))) {
        takes_string($value);
    } else {
        // @mago-expect analysis:type-confirmation
        \Mago\confirm($value, "string('default')");
    }

    return $value;
}

function test_assignment_in_if_body(?string $value): string
{
    if ($value !== null || ($value = '')) {
        $value = null;
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_nullable_assignment_in_if(?string $value): string
{
    if ($value === null && ($value = maybe_string()) !== null) {
        takes_string($value);
    }

    // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    return $value;
}

function test_or_in_if_with_null_assignment(?string $value): ?string
{
    // @mago-expect analysis:null-operand
    if ($value !== null || ($value = null)) {
        return takes_string($value);
    }

    // @mago-expect analysis:type-confirmation
    \Mago\confirm($value, 'null');

    return $value;
}

function test_nested_or_assignment_in_if(?string $value, bool $flag): ?string
{
    if ($value !== null || $flag && ($value = 'default')) {
        return takes_string($value);
    }

    return $value;
}

function test_nested_and_assignment_in_if(?string $value, bool $flag): ?string
{
    if ($value === null && ($flag && ($value = 'default'))) {
        return takes_string($value);
    }

    return $value;
}

/** @assert-if-true !null $value */
function has_string(?string $value): bool
{
    return $value !== null;
}

function test_rhs_assertion_after_assignment(?string $value, bool $flag): ?string
{
    if ($flag && has_string($value = maybe_string())) {
        return takes_string($value);
    }

    return $value;
}

final class NullableStringBox
{
    public function __construct(public ?string $value) {}
}

/** @assert-if-true =string $box->value */
function has_box_value(NullableStringBox $box): bool
{
    return $box->value !== null;
}

function test_overwritten_property_assertion(NullableStringBox $box): string
{
    if (has_box_value($box) && (($box = new NullableStringBox(null)) instanceof NullableStringBox)) {
        // @mago-expect analysis:nullable-return-statement,invalid-return-statement
        return $box->value;
    }

    return '';
}

function test_skipped_new_variable(bool $outer, bool $inner): void
{
    if ($outer && ($inner || ($value = maybe_string()))) {
        // @mago-expect analysis:undefined-variable,mixed-argument
        takes_string($value);
    }
}
