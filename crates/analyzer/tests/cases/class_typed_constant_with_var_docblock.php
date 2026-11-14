<?php

declare(strict_types=1);

enum Suit { case Hearts; case Spades; }

final class Deck
{
    /** @var Suit::Hearts */
    public const Suit FIRST = Suit::Hearts;
}

final class Table
{
    /** @var Suit::Spades|null */
    public const ?Suit DEFAULT = null;
}
