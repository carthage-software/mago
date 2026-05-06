<?php

declare(strict_types=1);

/**
 * @param array<array{
 *   validFrom: string,
 *   validTo: string|null,
 * }> $contracts
 */
function foo(array $contracts): array
{
    return array_filter($contracts, function (array $contract) {
        $validFromTimestamp = strtotime($contract['validFrom']);
        $validTo = $contract['validTo'] ?? 'foo';
        if ($validTo === null) {
            return false;
        }
        return true;
    });
}
