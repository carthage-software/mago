<?php

declare(strict_types=1);

function test_method_exists_ternary(object $obj): mixed
{
    return method_exists($obj, 'getValue') ? $obj->getValue() : null;
}

function test_property_exists_ternary(object $obj): mixed
{
    return property_exists($obj, 'value') ? $obj->value : null;
}

function test_function_exists_ternary(): mixed
{
    return function_exists('custom_func') ? custom_func() : null;
}

function test_defined_ternary(): mixed
{
    return defined('CUSTOM_CONST') ? CUSTOM_CONST : null;
}

function test_function_exists_null_coalesce(): mixed
{
    return function_exists('custom_func') ? custom_func() : null;
}

function test_match_function_exists(): mixed
{
    return match (true) {
        function_exists('custom_func') => custom_func(),
        default => null,
    };
}

function test_match_defined(): mixed
{
    return match (true) {
        defined('CUSTOM_CONST') => CUSTOM_CONST,
        default => null,
    };
}

function test_match_method_exists(object $obj): mixed
{
    return match (true) {
        method_exists($obj, 'getValue') => $obj->getValue(),
        default => null,
    };
}

function test_match_property_exists(object $obj): mixed
{
    return match (true) {
        property_exists($obj, 'value') => $obj->value,
        default => null,
    };
}

function test_switch_function_exists(): mixed
{
    switch (true) {
        case function_exists('custom_func'):
            return custom_func();
        default:
            return null;
    }
}

function test_switch_defined(): mixed
{
    switch (true) {
        case defined('CUSTOM_CONST'):
            return CUSTOM_CONST;
        default:
            return null;
    }
}

function test_switch_method_exists(object $obj): mixed
{
    switch (true) {
        case method_exists($obj, 'getValue'):
            return $obj->getValue();
        default:
            return null;
    }
}

function test_switch_property_exists(object $obj): mixed
{
    switch (true) {
        case property_exists($obj, 'value'):
            return $obj->value;
        default:
            return null;
    }
}

function test_function_exists_and_ternary(bool $flag): mixed
{
    return function_exists('custom_func') && $flag ? custom_func() : null;
}

function test_defined_and_ternary(bool $flag): mixed
{
    return defined('CUSTOM_CONST') && $flag ? CUSTOM_CONST : null;
}

function test_negated_function_exists_ternary(): mixed
{
    return !function_exists('custom_func') ? null : custom_func();
}

function test_negated_defined_ternary(): mixed
{
    return !defined('CUSTOM_CONST') ? null : CUSTOM_CONST;
}
