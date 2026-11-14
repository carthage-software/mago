<?php

declare(strict_types=1);

/** @param array{test: ?string} $a */
function f(array $a): void
{
    if ($a['test'] !== null) {
        echo 'set';
    } else {
        $a['test'] = 5;
    }

    if (is_int($a['test'])) {
        echo 'int';
    }
}

f(['test' => null]);
