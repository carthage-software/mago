<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:redundant-type-comparison
 */
function flow_redundant_is_int(int $v): bool
{
    return is_int($v);
}
