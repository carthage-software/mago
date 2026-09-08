<?php

declare(strict_types=1);

enum Suit
{
    case Hearts;
    case Spades;
}

/** @param Suit::* $suit */
function expectsCases(Suit $suit): void {}

/** @param Suit::Hearts|Suit::Spades $suit */
function expectsExplicitCases(Suit $suit): void {}

/** @param Suit::*|null $suit */
function expectsNullableCases(?Suit $suit): void {}

function passesNativeType(Suit $suit): void
{
    expectsCases($suit);
    expectsExplicitCases($suit);
}

function passesAfterComparison(Suit $suit): void
{
    if ($suit !== Suit::Hearts) {
        expectsCases($suit);
    }
}

function passesNullableType(?Suit $suit): void
{
    expectsNullableCases($suit);
}

/** @return Suit::* */
function returnsCases(Suit $suit): Suit
{
    return $suit;
}

/**
 * @template T of Suit
 *
 * @param T $suit
 */
function passesGenericType(Suit $suit): void
{
    expectsCases($suit);
}

enum Status: string
{
    case Active = 'active';
    case Inactive = 'inactive';

    public const LABEL = 'status';
}

/** @param Status::Active|Status::Inactive $status */
function expectsBackedCases(Status $status): void {}

/** @param Status::* $status */
function expectsCasesAndConstant(Status|string $status): void {}

function passesBackedType(Status $status): void
{
    expectsBackedCases($status);
    expectsCasesAndConstant($status);
    expectsCasesAndConstant(Status::LABEL);
}

enum Singleton
{
    case Only;
}

/** @param Singleton::Only $value */
function expectsOnlyCase(Singleton $value): void {}

function passesSingletonType(Singleton $value): void
{
    expectsOnlyCase($value);
}

enum Direction
{
    case North;
    case South;
    case East;
}

/** @param Direction::North|Direction::South $direction */
function expectsSubset(Direction $direction): void {}

function rejectsIncompleteUnion(Direction $direction): void
{
    // @mago-expect analysis:possibly-invalid-argument
    expectsSubset($direction);

    expectsSubset(Direction::North);
    expectsSubset(Direction::South);

    // @mago-expect analysis:invalid-argument
    expectsSubset(Direction::East);
}

enum OtherSuit
{
    case Hearts;
    case Spades;
}

function rejectsDifferentEnum(OtherSuit $suit): void
{
    // @mago-expect analysis:invalid-argument
    expectsCases($suit);
}

function rejectsUncoveredUnionMember(Suit|OtherSuit $suit): void
{
    // @mago-expect analysis:possibly-invalid-argument
    expectsCases($suit);
}

/** @param Suit::Hearts|OtherSuit::Spades $suit */
function expectsCasesFromDifferentEnums(Suit|OtherSuit $suit): void {}

function rejectsCasesFromDifferentEnums(Suit $suit): void
{
    // @mago-expect analysis:possibly-invalid-argument
    expectsCasesFromDifferentEnums($suit);
}
