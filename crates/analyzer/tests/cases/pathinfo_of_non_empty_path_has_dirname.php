<?php

declare(strict_types=1);

/**
 * @param non-empty-string $path
 */
function pathinfo_dirname_of_non_empty_path(string $path): string
{
    return pathinfo($path)['dirname'];
}

function pathinfo_dirname_of_literal_path(): string
{
    return pathinfo('a/b.c')['dirname'];
}

function pathinfo_dirname_of_any_path(string $path): string
{
    return pathinfo($path)['dirname'] ?? '.';
}
