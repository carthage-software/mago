<?php

declare(strict_types=1);

$singleLine = function (string $a, int $b): string
{
    return $a;
};

$multiLineWithReturnType = function (
    string $veryLongParameterNameThatExceedsWidth,
    int $anotherVeryLongParameterName,
    array $yetAnotherExtremelyLongParam,
): string
{
    return $veryLongParameterNameThatExceedsWidth;
};

$multiLineNoReturnType = function (
    string $veryLongParameterNameThatExceedsWidth,
    int $anotherVeryLongParameterName,
    array $yetAnotherExtremelyLongParam,
)
{
    return $veryLongParameterNameThatExceedsWidth;
};

$withUseClause = function (
    string $veryLongParameterNameThatExceedsWidth,
    int $anotherVeryLongParameterName,
) use ($someVar): string
{
    return $veryLongParameterNameThatExceedsWidth;
};
