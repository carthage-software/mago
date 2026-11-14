<?php

declare(strict_types=1);

function join_parts(): string
{
    $parts = ['a', 'b'];
    $parts[] = 'c';

    return $parts[0] . $parts[2];
}

/** @return list{string, string, string} */
function three(string $value): array
{
    $items = [$value, $value];
    $items[] = $value;

    return $items;
}

function past_the_end(): string
{
    $parts = ['a'];
    $parts[] = 'b';

    // @mago-expect analysis:undefined-int-array-index
    // @mago-expect analysis:null-operand
    return $parts[2] . '';
}
