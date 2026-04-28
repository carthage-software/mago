<?php

declare(strict_types=1);

/**
 * @return non-empty-list<int>
 */
function bad_empty(): array
{
    // @mago-expect analysis:invalid-return-statement
    return [];
}
