<?php

declare(strict_types=1);

/** @return array<array-key, mixed> */
function test_array_reduce_with_string_callable(): array
{
    return array_reduce(['a' => [1], 'b' => [2]], 'array_merge', []);
}

/** @return list<1|2> */
function test_array_reduce_with_closure(): array
{
    return array_reduce(
        ['a' => [1], 'b' => [2]],
        /**
         * @param list<1|2> $carry
         * @param list<1|2> $item
         * @return list<1|2>
         */
        function (array $carry, array $item): array {
            return array_merge($carry, $item);
        },
        [],
    );
}
