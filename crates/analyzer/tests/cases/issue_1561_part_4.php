<?php

declare(strict_types=1);

function super_upgrade_subsys(string $subsys): string
{
    $fn = $subsys . '_version';
    if (function_exists($fn)) {
        $fn();
    } else {
        echo "this else is reachable";
    }

    return 'done';
}
