<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:match-not-exhaustive
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_match_unhandled_int(int $code): string
{
    return match ($code) {
        200 => 'ok',
        404 => 'not found',
    };
}
