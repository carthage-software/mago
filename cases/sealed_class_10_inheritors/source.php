<?php

declare(strict_types=1);

/**
 * @psalm-inheritors Zero|One|Two|Three|Four|Five|Six|Seven|Eight|Nine
 */
abstract class Digit
{
    abstract public function getValue(): int;
}

class Zero extends Digit
{
    public function getValue(): int
    {
        return 0;
    }
}

class One extends Digit
{
    public function getValue(): int
    {
        return 1;
    }
}

class Two extends Digit
{
    public function getValue(): int
    {
        return 2;
    }
}

class Three extends Digit
{
    public function getValue(): int
    {
        return 3;
    }
}

class Four extends Digit
{
    public function getValue(): int
    {
        return 4;
    }
}

class Five extends Digit
{
    public function getValue(): int
    {
        return 5;
    }
}

class Six extends Digit
{
    public function getValue(): int
    {
        return 6;
    }
}

class Seven extends Digit
{
    public function getValue(): int
    {
        return 7;
    }
}

class Eight extends Digit
{
    public function getValue(): int
    {
        return 8;
    }
}

class Nine extends Digit
{
    public function getNextValue(): int
    {
        return 10;
    }

    public function getValue(): int
    {
        return 9;
    }
}

function describeDigit(Digit $digit): string
{
    if ($digit instanceof Zero) {
        return 'Zero: ' . $digit->getValue();
    }

    if ($digit instanceof One) {
        return 'One: ' . $digit->getValue();
    }

    if ($digit instanceof Two) {
        return 'Two: ' . $digit->getValue();
    }

    if ($digit instanceof Three) {
        return 'Three: ' . $digit->getValue();
    }

    if ($digit instanceof Four) {
        return 'Four: ' . $digit->getValue();
    }

    if ($digit instanceof Five) {
        return 'Five: ' . $digit->getValue();
    }

    if ($digit instanceof Six) {
        return 'Six: ' . $digit->getValue();
    }

    if ($digit instanceof Seven) {
        return 'Seven: ' . $digit->getValue();
    }

    if ($digit instanceof Eight) {
        return 'Eight: ' . $digit->getValue();
    }

    // After 9 checks, we know it must be Nine (boundary case: exactly 10 inheritors)
    return 'Nine: ' . $digit->getValue() . ' Next: ' . $digit->getNextValue();
}
