<?php

declare(strict_types=1);

/**
 * @param array<int,int> $curlist
 */
function test(array $curlist): void
{
    $my_rows = [];
    foreach ($curlist as $k => $v) {
        $my_rows[] = [
            'object_id' => $k,
            'score' => $v,
        ];
    }

    $my_index = 0;
    $my_generator = function ($_par) use (&$my_index, $my_rows): array|false {
        if (isset($my_rows[$my_index])) {
            return $my_rows[$my_index++];
        }
        return false;
    };
}
