<?php

declare(strict_types=1);

/**
 * @param list<list{string, int}> $rows
 */
function process(array $rows): void
{
    foreach ($rows as [$name, $age]) {
        echo $name, '=', $age;
    }
}
