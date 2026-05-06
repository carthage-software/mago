<?php

declare(strict_types=1);

/**
 * @return non-empty-list<string>
 */
function split_csv(string $s): array
{
    return explode(',', $s);
}
