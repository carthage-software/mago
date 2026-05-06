<?php

declare(strict_types=1);

enum State
{
    case Active;
    case Inactive;
}

/**
 */
function flow_match_unreachable_arm(State $s): string
{
    return match ($s) {
        State::Active => 'a',
        State::Inactive => 'i',
        State::Active => 'duplicate',
    };
}
