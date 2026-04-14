<?php

declare(strict_types=1);

namespace MagoBug;

/**
 * @return float|int|string|null
 */
function getLocale(): float|int|string|null
{
    return null;
}

function takesString(string $_value): void {}

function run(): void
{
    /** @var string[] $enabledLanguages */
    $enabledLanguages = ['de', 'en'];
    $requestedLocale = getLocale();

    if (
        $requestedLocale !== null &&
        in_array($requestedLocale, $enabledLanguages, strict: true)
    ) {
        takesString($requestedLocale);
    }
}
