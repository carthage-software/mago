<?php

declare(strict_types=1);

enum Mode
{
    case On;
    case Off;
}

/**
 */
function flow_match_redundant_arm(Mode $m): string
{
    return match ($m) {
        Mode::On => 'on',
        Mode::Off => 'off',
        default => 'other',
    };
}
