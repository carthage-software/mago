<?php

declare(strict_types=1);

enum Suit
{
    case Hearts;
    case Spades;
}

enum Rank
{
    case Ace;
}

/** @param Suit::* $suit */
function expectsCases(Suit $suit): void {}

function passesNativeType(Suit $suit): void
{
    expectsCases($suit);
}

function passesAfterComparison(Suit $suit): void
{
    if ($suit !== Suit::Hearts) {
        expectsCases($suit);
    }
}

function rejectsOtherEnum(Rank $rank): void
{
    // @mago-expect analysis:invalid-argument
    expectsCases($rank);
}
