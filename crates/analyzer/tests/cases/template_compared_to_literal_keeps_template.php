<?php

declare(strict_types=1);

/**
 * @template T of float
 *
 * @param T $value
 *
 * @return T
 */
function template_float_compared_to_literal(float $value): float
{
    if ($value === 1.5) {
        return $value;
    }

    return $value;
}

/**
 * @template T of class-string
 *
 * @param T $value
 *
 * @return T
 */
function template_class_string_compared_to_class_constant(string $value): string
{
    if ($value === stdClass::class) {
        return $value;
    }

    return $value;
}
