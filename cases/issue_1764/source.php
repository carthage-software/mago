<?php

declare(strict_types=1);

namespace test;

/**
 * @param array<int, int> $arr
 * @return array<int, int>
 */
function loop_unset(array $arr): array
{
    foreach ($arr as &$v) {
        $v += 10;
    }

    unset($v);

    return $arr;
}

function unset_after_conditional_assign(int $flag): void
{
    if ($flag > 0) {
        $x = 'set';
    }

    unset($x);
}

function unset_truly_undefined(): void
{
    unset($neverAssigned);
}
