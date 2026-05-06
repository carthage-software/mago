<?php

declare(strict_types=1);

function sub(int $oid): array
{
    return [mt_rand()];
}

/**
 * @param array<string,mixed> $o
 */
function main(int|string|array $o): int
{
    if (!is_array($o)) {
        $o = sub($o);
    }

    return mt_rand();
}
