<?php

/**
 * @param literal-float $a
 * @return literal-float
 */
function accept_literal_float(float $a): float
{
    return $a;
}

/**
 * @param float $a
 * @return float
 */
function accept_float(float $a): float
{
    return $a;
}

/**
 * @return literal-float
 */
function return_literal_float(): float
{
    return 3.14;
}

function test_pass_specific_literal_to_literal_float(): void
{
    $a = accept_literal_float(1.5);
    $b = accept_literal_float(2.71828);
    $c = accept_literal_float(0.0);
    $d = accept_literal_float(-1.5);
}

function test_pass_specific_literal_to_float(): void
{
    $a = accept_float(1.5);
    $b = accept_float(3.14159);
    $c = accept_float(0.0);
    $d = accept_float(-2.5);
}

function test_pass_literal_float_to_float(): void
{
    $a = return_literal_float();
    $b = accept_float($a);
}

function test_chain_literal_float_functions(): void
{
    $a = accept_float(accept_literal_float(1.5));
    $b = accept_float(return_literal_float());
}

/**
 * @param literal-float $x
 */
function test_literal_float_param_to_float(float $x): void
{
    accept_float($x);
}

/**
 * @return float
 */
function test_return_literal_float_as_float(): float
{
    return return_literal_float();
}

/**
 * @param literal-float $a
 * @param literal-float $b
 */
function test_multiple_literal_float_params(float $a, float $b): void
{
    accept_float($a);
    accept_float($b);
}

function test_call_with_multiple_literals(): void
{
    test_multiple_literal_float_params(1.5, 2.5);
}

/**
 * @param literal-float $x
 * @return float
 */
function test_literal_float_to_float_return(float $x): float
{
    return $x;
}

function test_nested_calls(): void
{
    $a = accept_float(accept_float(accept_literal_float(1.0)));
    $b = accept_literal_float(1.0);
    $c = accept_float($b);
}

/**
 * @param literal-float|null $x
 * @return literal-float|null
 */
function nullable_literal_float(?float $x): ?float
{
    return $x;
}

function test_nullable_literal_float(): void
{
    $a = nullable_literal_float(1.5);
    $b = nullable_literal_float(null);
}
