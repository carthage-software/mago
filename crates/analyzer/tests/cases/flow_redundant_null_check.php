<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:redundant-comparison
 */
function flow_redundant_null_check(string $s): bool
{
    return $s === null;
}
