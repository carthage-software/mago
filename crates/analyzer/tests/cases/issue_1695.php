<?php

declare(strict_types=1);

final class Thing
{
    public function check(): bool
    {
        return true;
    }
}

function nullsafe_narrow(?Thing $maybeThing): bool
{
    if (!$maybeThing?->check()) {
        exit(1);
    }

    return $maybeThing->check();
}

function ternary_narrow(?Thing $maybeThing): bool
{
    if (!($maybeThing !== null ? $maybeThing->check() : null)) {
        exit(1);
    }

    return $maybeThing->check();
}
