<?php

declare(strict_types=1);

/**
 * @return non-empty-array<string, int>
 */
function bad_empty(): array
{
    // @mago-expect analysis:invalid-return-statement
    return [];
}
