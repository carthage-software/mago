<?php

declare(strict_types=1);

function singleLine(string $a, int $b): string {
    return $a;
}

function multiLineWithReturnType(
    string $veryLongParameterNameThatExceedsWidth,
    int $anotherVeryLongParameterName,
    array $yetAnotherExtremelyLongParam,
): string {
    return $veryLongParameterNameThatExceedsWidth;
}
