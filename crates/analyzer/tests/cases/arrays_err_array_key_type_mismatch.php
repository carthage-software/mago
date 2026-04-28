<?php

declare(strict_types=1);

/**
 * @return array<string, int>
 */
function bad_keys(): array
{
    // @mago-expect analysis:invalid-return-statement
    return [0 => 1, 1 => 2];
}
