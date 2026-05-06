<?php

declare(strict_types=1);

/**
 * @param list<array{id: int, name: string}> $rows
 * @return list<string>
 */
function names(array $rows): array
{
    return array_column($rows, 'name');
}
