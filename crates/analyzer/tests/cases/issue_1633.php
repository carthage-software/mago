<?php

declare(strict_types=1);

class Foo
{
    public const VAL_1 = 1;

    public const DATA = [
        'something' => self::VAL_1,
    ];

    public function bar(int $key): void
    {
        $flipped = \array_flip(self::DATA);

        if (\array_key_exists($key, $flipped)) {
            echo $flipped[$key];
        }
    }
}

/** @var list<string> $list */
$list = ['a', 'b'];
$flipped_list = array_flip($list);
if (array_key_exists('a', $flipped_list)) {
    echo $flipped_list['a'];
}

/** @var array<string, int> $map */
$map = [];
$flipped_map = array_flip($map);
if (array_key_exists(0, $flipped_map)) {
    echo $flipped_map[0];
}

/**
 * @param array<1, non-empty-string> $a
 */
function extract_value(array $a, int $b): ?string
{
    if (array_key_exists($b, $a)) {
        return $a[$b];
    }

    return null;
}
