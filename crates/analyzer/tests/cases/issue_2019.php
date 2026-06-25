<?php

declare(strict_types=1);

enum Issue2019SomeEnum: string
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

$is1 = match (Issue2019SomeEnum::fromNumber(-1)) {
    Issue2019SomeEnum::Negative => 'negative',
    Issue2019SomeEnum::Positive => 'positive',
};

$is2 = match (Issue2019SomeEnum::Positive) {
    Issue2019SomeEnum::Negative => 'negative',
    Issue2019SomeEnum::Positive => 'positive',
};
