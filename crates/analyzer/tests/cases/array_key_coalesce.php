<?php

declare(strict_types=1);

/**
 * @param array<string, string> $data
 */
function processData(array $data): void
{
    if (null !== ($item = $data['item'] ?? null)) {
        echo 'Found item: ' . $item . "\n";
    }
}

/**
 * @param array<int, array{name?: string, value?: int}> $items
 */
function iterateWithDefaults(array $items): void
{
    foreach ($items as $key => $item) {
        $name = $item['name'] ?? 'Unknown';
        $value = $item['value'] ?? 0;

        echo "{$key}: {$name} = {$value}\n";
    }
}

/**
 * @param array<string, int> $config
 */
function getConfig(array $config, string $key): int
{
    return $config[$key] ?? 0;
}

function test(): void
{
    processData(['item' => 'test']);
    processData([]);

    iterateWithDefaults([
        ['name' => 'A', 'value' => 1],
        ['value' => 2],
        ['name' => 'C'],
    ]);

    $config = ['timeout' => 30, 'retries' => 3];
    echo getConfig($config, 'timeout') . "\n";
    echo getConfig($config, 'missing') . "\n";
}
