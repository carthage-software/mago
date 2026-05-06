<?php

declare(strict_types=1);

enum Suit
{
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

class MyClass
{
    public Suit $suit {
        get => $this->suit;
        set(Suit $value) => $value;
    }
}

class MyClass2
{
    public Suit $suit {
        get => $this->suit;
        set => $value;
    }
}
