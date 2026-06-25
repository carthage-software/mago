<?php

declare(strict_types=1);

enum SomEnum: string
{
    case Negative = 'negative';
    case Positive = 'positive';

    public static function fromNumber(int $number): self
    {
        return match (true) {
            $number < 0 => self::Negative,
            $number > 0 => self::Positive,
            default => throw new InvalidArgumentException('Number is zero.'),
        };
    }
}

$is1 = match (SomEnum::fromNumber(-1)) {
    SomEnum::Negative => 'negative',
    SomEnum::Positive => 'positive',
};

$is2 = match (SomEnum::Positive) {
    /** @mago-expect analysis:unreachable-match-arm */
    SomEnum::Negative => 'negative',
    SomEnum::Positive => 'positive',
};
