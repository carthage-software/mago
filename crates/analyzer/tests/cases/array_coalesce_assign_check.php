<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $params
 */
function processParams(array $params): int
{
    $params['page'] ??= 1;

    if (!is_int($params['page'])) {
        return 1;
    }

    return $params['page'];
}

/**
 * @param array<string, mixed> $queryParams
 * @return array<string, mixed>
 */
function processWithLimit(array $queryParams): array
{
    $queryParams['limit'] ??= 10;

    if (!is_int($queryParams['limit'])) {
        $queryParams['limit'] = 10;
    }

    if ($queryParams['limit'] < 1 || $queryParams['limit'] > 100) {
        $queryParams['limit'] = 10;
    }

    return $queryParams;
}

function test(): void
{
    $result1 = processParams(['page' => 5]);
    $result2 = processParams([]);
    $result3 = processParams(['page' => 'invalid']);
}
