<?php

declare(strict_types=1);

/**
 * @param list<string> $items
 */
function flow_loop_with_break_then_use(array $items): null|string
{
    $found = null;

    foreach ($items as $item) {
        if ($item !== '') {
            $found = $item;
            break;
        }
    }

    return $found;
}
