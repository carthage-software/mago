<?php

declare(strict_types=1);

/**
 * @param literal-string $sql
 */
function string_refinement_query(string $sql): void
{
    echo $sql;
}

/**
 * @param literal-string|int $a
 * @param literal-string $b
 */
function string_refinement_run(string|int $a, string $b): void
{
    if (is_string($a)) {
        string_refinement_query($a);
    }

    if ($b !== '') {
        string_refinement_query($b);
    }
}

/**
 * @param uppercase-string $code
 */
function string_refinement_takes_upper(string $code): void
{
    echo $code;
}

/**
 * @param uppercase-string $upper
 * @param lowercase-string $lower
 */
function string_refinement_compare(string $upper, string $lower): void
{
    if ($upper === $lower) {
        string_refinement_takes_upper($upper);
    }
}
