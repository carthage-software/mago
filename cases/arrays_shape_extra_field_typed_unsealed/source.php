<?php

declare(strict_types=1);

/**
 * @return array{a: int, ...<string, string>}
 */
function bad_extra(): array
{
    return ['a' => 1, 'foo' => 42];
}
