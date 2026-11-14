<?php

declare(strict_types=1);

namespace FullyQualifiedExistenceGuards;

function call_guarded(): void
{
    if (\function_exists('some_optional_function')) {
        \some_optional_function();
    }

    if (function_exists('\other_optional_function')) {
        \other_optional_function();
    }
}

function read_guarded(): mixed
{
    if (\defined('SOME_OPTIONAL_CONSTANT')) {
        return \SOME_OPTIONAL_CONSTANT;
    }

    if (defined('\OTHER_OPTIONAL_CONSTANT')) {
        return \OTHER_OPTIONAL_CONSTANT;
    }

    return null;
}
