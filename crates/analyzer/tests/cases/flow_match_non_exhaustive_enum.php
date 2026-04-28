<?php

declare(strict_types=1);

enum Light
{
    case Red;
    case Yellow;
    case Green;
}

/**
 * @mago-expect analysis:match-not-exhaustive
 * @mago-expect analysis:unhandled-thrown-type
 */
function flow_match_non_exhaustive_enum(Light $l): string
{
    return match ($l) {
        Light::Red => 'stop',
        Light::Green => 'go',
    };
}
