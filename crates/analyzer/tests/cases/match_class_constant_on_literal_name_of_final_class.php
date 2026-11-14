<?php

declare(strict_types=1);

final class Kind
{
    public function viaParameter(Kind $k): string
    {
        return match ($k::class) {
            'Kind' => 'yes',
        };
    }

    public function viaVar(mixed $m): string
    {
        /** @var Kind $s */
        $s = $m;

        return match ($s::class) {
            'Kind' => 'yes',
        };
    }
}

enum Suit
{
    case Hearts;
}

function suit(Suit $suit): string
{
    return match ($suit::class) {
        'Suit' => 'suit',
    };
}

class Open
{
}

function open(Open $open): string
{
    return match ($open::class) {
        'Open' => 'exactly open',
        default => 'a subclass',
    };
}
