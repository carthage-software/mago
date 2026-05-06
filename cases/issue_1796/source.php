<?php

declare(strict_types=1);

/** @param array{string, int, foo?: string} $data */
function foo(array $data): void
{
    [$first, $second] = $data;
    echo $first;
    echo $second;
}
