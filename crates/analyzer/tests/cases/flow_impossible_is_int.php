<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:impossible-type-comparison
 */
function flow_impossible_is_int(string $v): bool
{
    return is_int($v);
}
