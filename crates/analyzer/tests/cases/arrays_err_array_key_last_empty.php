<?php

declare(strict_types=1);

/**
 * @param array<string, int> $arr
 */
function bad(array $arr): string
{
    // @mago-expect analysis:invalid-return-statement,nullable-return-statement
    return array_key_last($arr);
}
