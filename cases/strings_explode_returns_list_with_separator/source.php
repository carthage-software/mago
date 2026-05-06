<?php

declare(strict_types=1);

/**
 * @return list<string>
 */
function probe(string $s): array
{
    return explode(',', $s);
}
