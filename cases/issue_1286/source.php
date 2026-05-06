<?php

declare(strict_types=1);

enum Suit
{
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

/**
 */
class Card
{
    public Suit $suit;
    public string $label;
}

function apply_defaults(Card $card): void
{
    $card->suit ??= Suit::Hearts;
    $card->label ??= 'Unknown';
}

function read_with_fallback(Card $card): string
{
    $label = $card->label ?? 'Fallback';

    return $label;
}

// Properties with defaults: ?? IS redundant
class WithDefaults
{
    public string $name = 'default';
    public int $count = 0;
}

function test_with_defaults(WithDefaults $obj): void
{
    $_ = $obj->name ?? 'fallback';
    $_ = $obj->count ?? 42;
}

class Untyped
{
    /** @var string */
    public $name = 'default';
}

function test_untyped(Untyped $obj): void
{
    $_ = $obj->name ?? 'fallback';
}

class Promoted
{
    public function __construct(
        public string $name,
        public int $age,
    ) {}
}

function test_promoted(Promoted $obj): void
{
    $_ = $obj->name ?? 'fallback';
    $_ = $obj->age ?? 0;
}
