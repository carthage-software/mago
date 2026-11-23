<?php

declare(strict_types=1);

final class DebugTypeTarget
{
    public function run(): void
    {
    }
}

function get_debug_type_narrows_class(DebugTypeTarget|int|null $value): void
{
    if (get_debug_type($value) === DebugTypeTarget::class) {
        $value->run();
    }
}

function get_debug_type_narrows_scalar(DebugTypeTarget|int|null $value): int
{
    if (get_debug_type($value) === 'int') {
        return $value;
    }

    return 0;
}

function get_debug_type_negation_narrows(DebugTypeTarget|null $value): void
{
    if (get_debug_type($value) !== 'null') {
        $value->run();
    }
}

/**
 * @return array<array-key, mixed>
 */
function gettype_array_narrows(mixed $value): array
{
    if (gettype($value) === 'array') {
        return $value;
    }

    return [];
}
