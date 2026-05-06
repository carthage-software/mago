<?php

declare(strict_types=1);

/**
 * @template T
 * @param T $out
 * @param-out T $out
 */
function event_run(mixed &$out = ''): int
{
    if (mt_rand(0, max: 10) === 0) {
        $out .= 'text';
    }

    return mt_rand(0, max: 1);
}

$out = [
    'fatalmsg' => '',
    'warnmsg' => '',
];

event_run($out);

if ($out['warnmsg']) {
    echo $out['warnmsg'];
}
