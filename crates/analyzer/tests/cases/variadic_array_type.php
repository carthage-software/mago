<?php

/**
 * @return list<string>
 *
 * @no-named-arguments
 */
function test1(string ...$items): array
{
    if (array_is_list($items)) { // @mago-expect analysis:redundant-type-comparison,redundant-condition
        echo 'it is a list, no named args were used!';
    }

    return $items;
}

/**
 * @return array<array-key, string>
 */
function test2(string ...$items): array
{
    if (array_is_list($items)) {
        echo 'it is a list, no named args were used!';
    }

    return $items;
}

test1('a', 'b', 'c');
test2('a', 'b', 'c');
test2(a: 'a', b: 'b', c: 'c');
