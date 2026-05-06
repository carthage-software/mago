<?php

declare(strict_types=1);

/**
 * @template T
 * @param T $out
 * @param-out T $out
 */
function event_run(mixed &$out = ''): int
{
    if (mt_rand(0, 10) === 0) {
        $out .= 'text';
    }

    return mt_rand(0, 1);
}

$t = '';
event_run($t);
if ($t) {
    echo "the event did something\n";
}
