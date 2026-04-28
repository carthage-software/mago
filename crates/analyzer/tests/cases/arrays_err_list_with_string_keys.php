<?php

declare(strict_types=1);

/**
 * @return list<int>
 */
function bad_list(): array
{
    // @mago-expect analysis:invalid-return-statement
    return ['a' => 1, 'b' => 2];
}
