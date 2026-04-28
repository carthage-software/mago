<?php

declare(strict_types=1);

/**
 * @phpstan-type StringMap array<string, int>
 */
final class CounterMap
{
    /**
     * @param StringMap $m
     * @return StringMap
     */
    public function increment(array $m, string $key): array
    {
        $m[$key] = ($m[$key] ?? 0) + 1;

        return $m;
    }
}

$c = new CounterMap();
$out = $c->increment(['a' => 1, 'b' => 2], 'a');
echo $out['a'] ?? 0;
