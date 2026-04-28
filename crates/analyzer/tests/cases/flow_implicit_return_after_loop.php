<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 */
function flow_implicit_return_after_loop(array $items): null|int
{
    foreach ($items as $i) {
        if ($i > 0) {
            return $i;
        }
    }

    return null;
}
