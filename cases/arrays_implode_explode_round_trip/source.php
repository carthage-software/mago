<?php

declare(strict_types=1);

/**
 * @param list<string> $xs
 */
function join_then_split(array $xs): string
{
    $joined = implode(',', $xs);
    return $joined;
}
