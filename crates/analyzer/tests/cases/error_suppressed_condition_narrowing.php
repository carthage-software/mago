<?php

declare(strict_types=1);

/**
 * @param array<array-key, mixed> $values
 */
function suppressed_class_exists(array $values): void
{
    foreach ($values as $name => $_value) {
        if (!is_string($name)) {
            continue;
        }

        if (@class_exists($name)) {
            takes_class_string($name);
        }
    }
}

function suppressed_is_string(mixed $value): string
{
    if (@is_string($value)) {
        return $value;
    }

    return '';
}

function suppressed_negated_is_string(mixed $value): string
{
    if (!@is_string($value)) {
        return '';
    }

    return $value;
}

/**
 * @param class-string $class
 */
function takes_class_string(string $class): void
{
    echo $class;
}
