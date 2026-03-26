<?php

declare(strict_types=1);

function test_array_filter_nullable(): void
{
    /** @var list<string|null> $paths */
    $paths = ['a', null, 'b'];
    $filtered = array_values(array_filter($paths, static fn($v) => $v !== null));
}
