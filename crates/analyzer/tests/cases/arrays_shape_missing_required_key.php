<?php

declare(strict_types=1);

/**
 * @return array{a: int, b: string}
 */
function bad_shape(): array
{
    // @mago-expect analysis:invalid-return-statement
    return ['a' => 1];
}
