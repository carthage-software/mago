<?php

declare(strict_types=1);

/**
 * @param non-falsy-string $s
 */
function flow_redundant_truthy(string $s): string
{
    /** @mago-expect analysis:redundant-condition */
    if ($s) {
        return $s;
    }

    return '';
}
