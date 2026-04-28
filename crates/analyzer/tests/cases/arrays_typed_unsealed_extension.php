<?php

declare(strict_types=1);

/**
 * @return array{a: int, ...<string, mixed>}
 */
function typed_unsealed(): array
{
    return ['a' => 1, 'foo' => 'bar', 'baz' => 42];
}
