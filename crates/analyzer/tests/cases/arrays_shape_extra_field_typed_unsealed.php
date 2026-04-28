<?php

declare(strict_types=1);

/**
 * @return array{a: int, ...<string, string>}
 */
function bad_extra(): array
{
    // @mago-expect analysis:invalid-return-statement
    return ['a' => 1, 'foo' => 42];
}
