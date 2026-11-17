<?php

declare(strict_types=1);

function loop_variable_while(int $u): void
{
    while ($u > 0) {
        $u--;
        if ($u === 5) {
            $q = 1;
            break;
        }

        $q = 'a';
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if ($q === 'a') {
        echo 'a';
    }
}

/**
 * @param list<int> $items
 */
function loop_variable_foreach(array $items): void
{
    foreach ($items as $item) {
        if ($item === 5) {
            $found = 1;
            break;
        }

        $found = 'none';
    }

    /** @mago-expect analysis:possibly-undefined-variable */
    if ($found === 'none') {
        echo 'none';
    }
}
