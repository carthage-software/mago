<?php

declare(strict_types=1);

/**
 * @param list<string> $xs
 * @return array<int, string>
 */
function dedup_strings(array $xs): array
{
    return array_unique($xs);
}
