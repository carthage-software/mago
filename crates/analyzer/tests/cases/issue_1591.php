<?php

declare(strict_types=1);

/** @return list<array{currency: string, ratio: string}> */
function loadRates(): array
{
    return [['ratio' => '123', 'currency' => 'USD']];
}

/** @return array<string, string> */
function getRates(): array
{
    return array_column(loadRates(), 'ratio', 'currency');
}

/** @return list<string> */
function getRatios(): array
{
    return array_column(loadRates(), 'ratio');
}

/** @return list<array{currency: string, ratio: string}> */
function getRows(): array
{
    return array_column(loadRates(), null);
}

/** @return array<int, string> */
function getByInt(): array
{
    /** @var list<array{id: int, name: string}> $items */
    $items = [];

    return array_column($items, 'name', 'id');
}
