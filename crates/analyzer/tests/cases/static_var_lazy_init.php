<?php

declare(strict_types=1);

class ExpensiveService
{
    public function __construct()
    {
        // Expensive initialization
    }

    public function process(string $input): string
    {
        return strtoupper($input);
    }
}

function getService(): ExpensiveService
{
    static $service = null;

    if ($service === null) {
        $service = new ExpensiveService();
    }

    return $service;
}

/**
 * Another common pattern with null check.
 *
 * @var array<string, string>|null $cache
 */
function getCachedValue(string $key): string
{
    /** @var array<string, string>|null $cache */
    static $cache = null;

    if (null === $cache) {
        $cache = [];
    }

    if (!isset($cache[$key])) {
        $cache[$key] = 'computed_' . $key;
    }

    return $cache[$key];
}

function test(): void
{
    $service1 = getService();
    $service2 = getService();

    echo $service1->process('hello');
    echo getCachedValue('foo');
}
