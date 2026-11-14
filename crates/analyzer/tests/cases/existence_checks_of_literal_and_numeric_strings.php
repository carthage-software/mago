<?php

declare(strict_types=1);

/**
 * @param class-string $name
 */
function existence_takes_class_string(string $name): string
{
    return $name;
}

/**
 * @param callable-string $name
 */
function existence_takes_callable_string(string $name): string
{
    return $name;
}

/**
 * @param literal-string $name
 */
function existence_class_of_literal(string $name): string
{
    if (class_exists($name)) {
        return existence_takes_class_string($name);
    }

    return '';
}

/**
 * @param literal-string $name
 */
function existence_interface_of_literal(string $name): string
{
    if (interface_exists($name)) {
        return strtoupper($name);
    }

    return '';
}

/**
 * @param non-empty-literal-string $name
 */
function existence_enum_and_trait_of_literal(string $name): string
{
    if (enum_exists($name) || trait_exists($name)) {
        return existence_takes_class_string($name);
    }

    return '';
}

/**
 * @param non-empty-literal-string $name
 */
function existence_function_of_literal(string $name): string
{
    if (function_exists($name)) {
        return existence_takes_callable_string($name);
    }

    return '';
}

/**
 * @param numeric-string $name
 */
function existence_function_of_numeric(string $name): string
{
    if (function_exists($name)) {
        // @mago-expect analysis:no-value
        return existence_takes_callable_string($name);
    }

    return '';
}
