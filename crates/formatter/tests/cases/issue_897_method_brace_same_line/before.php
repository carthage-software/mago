<?php

declare(strict_types=1);

class Foo
{
    public function singleLine(string $a, int $b): string
    {
        return $a;
    }

    public function multiLineWithReturnType(
        string $veryLongParameterNameThatExceedsWidth,
        int $anotherVeryLongParameterName,
        array $yetAnotherExtremelyLongParam,
    ): string
    {
        return $veryLongParameterNameThatExceedsWidth;
    }
}
