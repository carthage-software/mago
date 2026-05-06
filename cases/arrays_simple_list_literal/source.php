<?php

declare(strict_types=1);

/**
 * @return list<int>
 */
function build_list(): array
{
    return [1, 2, 3];
}

/**
 * @return non-empty-list<int>
 */
function build_non_empty_list(): array
{
    return [1, 2, 3];
}
