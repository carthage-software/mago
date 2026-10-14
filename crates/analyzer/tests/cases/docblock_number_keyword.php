<?php

declare(strict_types=1);

final class NumberProvider
{
    /** @return number */
    public function amount(): int|float
    {
        return 1.5;
    }

    /**
     * @param NUMBER $value
     *
     * @return number
     */
    public function twice(int|float $value): int|float
    {
        return $value * 2;
    }
}

echo (new NumberProvider())->amount();
echo (new NumberProvider())->twice(21);
