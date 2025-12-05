<?php

declare(strict_types=1);

class LazyCache
{
    /** @var array<string, string>|null */
    private null|array $cache = null;

    /** @var string|null */
    private null|string $defaultValue = null;

    public function getCached(string $key): string
    {
        $this->cache ??= [];

        if (!isset($this->cache[$key])) {
            $this->cache[$key] = $this->computeExpensive($key);
        }

        return $this->cache[$key];
    }

    private function computeExpensive(string $key): string
    {
        return 'computed_' . $key;
    }

    public function getDefault(): string
    {
        $this->defaultValue ??= 'default';

        return $this->defaultValue;
    }

    public function clearCache(): void
    {
        $this->cache = null;
    }
}

function test(): void
{
    $cache = new LazyCache();
    echo $cache->getCached('foo') . "\n";
    echo $cache->getCached('foo') . "\n";

    echo $cache->getDefault() . "\n";

    $cache->clearCache();
    echo $cache->getCached('foo') . "\n";
}
