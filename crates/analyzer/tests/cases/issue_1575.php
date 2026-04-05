<?php

declare(strict_types=1);

/** @return array<string,int> */
function fassoc(): array
{
    return [
        'object_id' => mt_rand(1, max: 100_000),
    ];
}

function object_url(int $oid): string
{
    return (string) $oid;
}

/**
 * @return array<string>
 */
function test(int $mon, int $dom, bool $cheat): array
{
    $list = [];
    while ($row = fassoc()) {
        /** @mago-expect analysis:possibly-undefined-string-array-index */
        $list[] = $row['object_id'];
    }

    $out = [];
    for ($i = 1; $i < 25; $i++) {
        if (!$cheat && 12 === $mon && $i > $dom) {
            continue;
        }
        if (!array_key_exists($i, $list)) {
            continue;
        }

        $oid = $list[$i - 1];
        $out[] = object_url($oid);
    }

    return $out;
}
