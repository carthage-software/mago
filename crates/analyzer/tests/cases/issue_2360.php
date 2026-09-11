<?php

declare(strict_types=1);

namespace Issue2360;

enum Kind: string
{
    case A = 'a';
    case B = 'b';
    case C = 'c';
}

enum Label: string
{
    case X = 'x';
    case Y = 'y';
}

final readonly class Repro
{
    public static function mapEnumOnly(Kind $kind): string
    {
        return match ($kind) {
            Kind::A => 'a',
            Kind::B => 'b',
            Kind::C => 'c',
        };
    }

    public static function mapPair(Kind $kind, bool $flag): string
    {
        return match ([$kind, $flag]) {
            [Kind::A, true] => 'a-with',
            [Kind::A, false] => 'a-without',
            [Kind::B, true] => 'b-with',
            [Kind::B, false] => 'b-without',
            [Kind::C, true],
            [Kind::C, false] => 'c',
        };
    }

    public static function mapTriple(Kind $kind, bool $flag, Label $label): string
    {
        return match ([$kind, $flag, $label]) {
            [Kind::A, true, Label::X] => 'a-t-x',
            [Kind::A, true, Label::Y] => 'a-t-y',
            [Kind::A, false, Label::X] => 'a-f-x',
            [Kind::A, false, Label::Y] => 'a-f-y',
            [Kind::B, true, Label::X] => 'b-t-x',
            [Kind::B, true, Label::Y] => 'b-t-y',
            [Kind::B, false, Label::X] => 'b-f-x',
            [Kind::B, false, Label::Y] => 'b-f-y',
            [Kind::C, true, Label::X] => 'c-t-x',
            [Kind::C, true, Label::Y] => 'c-t-y',
            [Kind::C, false, Label::X] => 'c-f-x',
            [Kind::C, false, Label::Y] => 'c-f-y',
        };
    }

    public static function mapAssoc(Kind $kind, bool $flag): string
    {
        return match (['kind' => $kind, 'flag' => $flag]) {
            ['kind' => Kind::A, 'flag' => true] => 'a-with',
            ['kind' => Kind::A, 'flag' => false] => 'a-without',
            ['kind' => Kind::B, 'flag' => true] => 'b-with',
            ['kind' => Kind::B, 'flag' => false] => 'b-without',
            ['kind' => Kind::C, 'flag' => true],
            ['kind' => Kind::C, 'flag' => false] => 'c',
        };
    }

    /**
     * @mago-expect analysis:match-not-exhaustive
     * @mago-expect analysis:unhandled-thrown-type
     */
    public static function mapIncompletePair(Kind $kind, bool $flag): string
    {
        return match ([$kind, $flag]) {
            [Kind::A, true] => 'a-with',
            [Kind::A, false] => 'a-without',
            [Kind::B, true] => 'b-with',
            [Kind::B, false] => 'b-without',
            [Kind::C, true] => 'c-with',
        };
    }

    /**
     * @mago-expect analysis:match-not-exhaustive
     * @mago-expect analysis:unhandled-thrown-type
     */
    public static function mapReorderedAssoc(Kind $kind, bool $flag): string
    {
        return match (['kind' => $kind, 'flag' => $flag]) {
            ['kind' => Kind::A, 'flag' => true] => 'a-with',
            ['kind' => Kind::A, 'flag' => false] => 'a-without',
            ['kind' => Kind::B, 'flag' => true] => 'b-with',
            ['kind' => Kind::B, 'flag' => false] => 'b-without',
            ['kind' => Kind::C, 'flag' => true] => 'c-with',
            ['flag' => false, 'kind' => Kind::C] => 'c-without',
        };
    }
}
