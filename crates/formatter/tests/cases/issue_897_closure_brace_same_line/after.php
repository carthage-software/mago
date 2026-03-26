<?php

declare(strict_types=1);

$singleLine = function (string $a, int $b): string {
    return $a;
};

$multiLineWithReturnType = function (
    string $veryLongParameterNameThatExceedsWidth,
    int $anotherVeryLongParameterName,
    array $yetAnotherExtremelyLongParam,
): string {
    return $veryLongParameterNameThatExceedsWidth;
};
