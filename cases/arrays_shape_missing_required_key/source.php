<?php

declare(strict_types=1);

/**
 * @return array{a: int, b: string}
 */
function bad_shape(): array
{
    return ['a' => 1];
}
