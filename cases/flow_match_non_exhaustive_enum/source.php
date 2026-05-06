<?php

declare(strict_types=1);

enum Light
{
    case Red;
    case Yellow;
    case Green;
}

/**
 */
function flow_match_non_exhaustive_enum(Light $l): string
{
    return match ($l) {
        Light::Red => 'stop',
        Light::Green => 'go',
    };
}
