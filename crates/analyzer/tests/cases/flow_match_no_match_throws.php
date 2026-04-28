<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:match-not-exhaustive
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_match_no_match_throws(int $v): string
{
    return match ($v) {
        1 => 'one',
        2 => 'two',
    };
}
