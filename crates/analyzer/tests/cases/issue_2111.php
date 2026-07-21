<?php

declare(strict_types=1);

enum MyEnum: int
{
    case TwoPi = 2 * self::PI;
    case Pi = 0 + self::PI;
    case PiMod2 = self::PI % 2;

    private const int PI = 3;
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if (MyEnum::TwoPi->value === 6) {
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if (MyEnum::Pi->value === 3) {
}

// @mago-expect analysis:redundant-comparison,redundant-condition
if (MyEnum::PiMod2->value === 1) {
}
