<?php

declare(strict_types=1);

/**
 * @return array{a: int, b: string}
 */
function wrong_value_type(): array
{
    return ['a' => 1, 'b' => 2];
}
