<?php

declare(strict_types=1);

function test_method_exists(object $obj): mixed
{
    if (method_exists($obj, 'getValue')) {
        return $obj->getValue();
    }

    return null;
}

function test_property_exists(object $obj): mixed
{
    if (property_exists($obj, 'value')) {
        return $obj->value;
    }

    return null;
}

function test_method_exists_with_negation(object $obj): mixed
{
    if (!method_exists($obj, 'getValue')) {
        return null;
    }

    return $obj->getValue();
}

function test_property_exists_with_negation(object $obj): mixed
{
    if (!property_exists($obj, 'value')) {
        return null;
    }

    return $obj->value;
}

function test_method_exists_and_condition(object $obj, bool $flag): mixed
{
    if (method_exists($obj, 'getValue') && $flag) {
        return $obj->getValue();
    }

    return null;
}

function test_property_exists_and_condition(object $obj, bool $flag): mixed
{
    if (property_exists($obj, 'value') && $flag) {
        return $obj->value;
    }

    return null;
}

function test_function_exists(): mixed
{
    if (function_exists('custom_debug_log')) {
        return custom_debug_log('message');
    }

    return null;
}

function test_function_exists_with_negation(): mixed
{
    if (!function_exists('custom_debug_log')) {
        return null;
    }

    return custom_debug_log('message');
}

function test_defined(): mixed
{
    if (defined('CUSTOM_DEBUG_MODE')) {
        return CUSTOM_DEBUG_MODE;
    }

    return null;
}

function test_defined_with_negation(): mixed
{
    if (!defined('CUSTOM_DEBUG_MODE')) {
        return null;
    }

    return CUSTOM_DEBUG_MODE;
}
