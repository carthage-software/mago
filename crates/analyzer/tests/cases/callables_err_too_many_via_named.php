<?php

declare(strict_types=1);

function callables_named_target_two(string $first, int $second): string
{
    return $first . $second;
}

callables_named_target_two(first: 'x', second: 1, third: 'no'); // @mago-expect analysis:too-many-arguments,invalid-named-argument
