<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param T $value
 *
 * @return list<T>
 */
function gen_repeat2(mixed $value, int $n): array
{
    $r = [];
    for ($i = 0; $i < $n; $i++) {
        $r[] = $value;
    }
    return $r;
}

/**
 * @param list<string> $a
 */
function takes_list_str(array $a): void
{
    foreach ($a as $s) {
        echo $s;
    }
}

takes_list_str(gen_repeat2('x', 3));
