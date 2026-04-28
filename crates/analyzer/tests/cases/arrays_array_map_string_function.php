<?php

declare(strict_types=1);

/**
 * @param list<string> $xs
 * @return list<string>
 */
function upper_all(array $xs): array
{
    return array_map('strtoupper', $xs);
}
