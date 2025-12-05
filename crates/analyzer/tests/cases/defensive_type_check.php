<?php

declare(strict_types=1);

/**
 * @param array{count?: int, name?: string, enabled?: bool} $data
 * @return array{count: int, name: string, enabled: bool}
 */
function processExternalData(array $data): array
{
    return [
        'count' => $data['count'] ?? 0,
        'name' => $data['name'] ?? '',
        'enabled' => $data['enabled'] ?? false,
    ];
}

/**
 * @return array{id: int, status: string}
 */
function loadFromDatabase(int $id): array
{
    $row = fetchRow($id);

    if ($row === null) {
        return ['id' => $id, 'status' => 'unknown'];
    }

    return [
        'id' => $row['id'],
        'status' => $row['status'],
    ];
}

/**
 * @return array{id: int, status: string}|null
 */
function fetchRow(int $id): null|array
{
    return ['id' => $id, 'status' => 'active'];
}

/**
 * @param array{debug?: bool, timeout?: int, retries?: int} $config
 * @return array{debug: bool, timeout: int, retries: int}
 */
function validateConfig(array $config): array
{
    $debug = $config['debug'] ?? false;
    $timeout = $config['timeout'] ?? 30;
    $retries = $config['retries'] ?? 3;

    if ($timeout < 0) {
        $timeout = 30;
    }

    if ($retries < 0) {
        $retries = 3;
    }

    return [
        'debug' => $debug,
        'timeout' => $timeout,
        'retries' => $retries,
    ];
}

function test(): void
{
    $data = processExternalData(['count' => 10, 'name' => 'test']);
    echo "Count: {$data['count']}\n";

    $row = loadFromDatabase(1);
    echo "ID: {$row['id']}, Status: {$row['status']}\n";

    $config = validateConfig(['debug' => true, 'timeout' => 60]);
    echo "Timeout: {$config['timeout']}\n";
}
