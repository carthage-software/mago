<?php

declare(strict_types=1);

/**
 * @return array<string, int>
 */
function bad_values(): array
{
    // @mago-expect analysis:invalid-return-statement
    return ['a' => 'not an int'];
}
