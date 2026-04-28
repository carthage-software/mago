<?php

declare(strict_types=1);

function example(int $count): int {
    $i = 0;
    while ($i < $count) {
        $i++;
    }
    return $i;
}

example(5);
