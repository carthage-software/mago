<?php

declare(strict_types=1);

enum State
{
    case Active;
    case Inactive;
}

/**
 * @mago-expect analysis:match-arm-always-true
 * @mago-expect analysis:unreachable-match-arm
 */
function flow_match_unreachable_arm(State $s): string
{
    return match ($s) {
        State::Active => 'a',
        State::Inactive => 'i',
        State::Active => 'duplicate',
    };
}
