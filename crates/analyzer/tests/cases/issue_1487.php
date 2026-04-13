<?php

declare(strict_types=1);

function sub(int $oid): array
{
    return [mt_rand()];
}

/**
 * @param array<string,mixed> $o
 */
function main(int|string|array $o): int // @mago-expect analysis:docblock-parameter-narrowing
{
    // @mago-expect analysis:redundant-type-comparison
    // @mago-expect analysis:impossible-condition
    if (!is_array($o)) {
        $o = sub($o); // @mago-expect analysis:no-value
    }

    return mt_rand();
}
