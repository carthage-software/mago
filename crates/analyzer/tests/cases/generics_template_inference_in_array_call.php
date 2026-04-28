<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @param list<T> $list
 *
 * @return list<T>
 */
function gen_dup(array $list): array
{
    return [...$list, ...$list];
}

/** @var list<int> $arr */
$arr = [1, 2, 3];
$out = gen_dup($arr);
foreach ($out as $n) {
    echo $n + 1;
}
