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
 * @mago-expect analysis:missing-constructor
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
    // @mago-expect analysis:redundant-null-coalesce
    $_ = $obj->name ?? 'fallback';
    // @mago-expect analysis:redundant-null-coalesce
    $_ = $obj->count ?? 42;
}

class Untyped
{
    /** @var string */
    public $name = 'default';
}

function test_untyped(Untyped $obj): void
{
    // @mago-expect analysis:redundant-null-coalesce
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
    // @mago-expect analysis:redundant-null-coalesce
    $_ = $obj->name ?? 'fallback';
    // @mago-expect analysis:redundant-null-coalesce
    $_ = $obj->age ?? 0;
}
