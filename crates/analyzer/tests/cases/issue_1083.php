<?php

declare(strict_types=1);

/** @mago-ignore linter:file-name */
class Bar
{
    public function __construct(
        public readonly int $value,
    ) {}
}

/**
 * @param non-empty-list<Bar> $foo
 */
function get_optimal(array $foo): Bar
{
    usort($foo, fn(Bar $a, Bar $b): int => $a->value <=> $b->value);
    $bar = array_shift($foo);

    return $bar;
}
