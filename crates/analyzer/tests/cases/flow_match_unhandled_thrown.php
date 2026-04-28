<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:match-not-exhaustive
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_match_unhandled_thrown(string $s): int
{
    return match ($s) {
        'a' => 1,
        'b' => 2,
    };
}
