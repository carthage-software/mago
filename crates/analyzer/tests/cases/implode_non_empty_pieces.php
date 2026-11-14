<?php

declare(strict_types=1);

/** @param non-empty-string $s */
function implode_takes_non_empty(string $s): void
{
    echo $s;
}

function implode_non_empty_pieces(string $dir, int $i): void
{
    implode_takes_non_empty(implode('/', [$dir, 'file.txt']));
    implode_takes_non_empty(implode('', [$dir, 'file.txt']));
    implode_takes_non_empty(implode('/', [$dir, $dir]));
    implode_takes_non_empty(implode('', [$dir, $i]));
    // @mago-expect analysis:possibly-invalid-argument
    implode_takes_non_empty(implode('/', [$dir]));
}
