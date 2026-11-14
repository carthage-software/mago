<?php

declare(strict_types=1);

/**
 * @param non-empty-list<int> $a
 */
function non_empty_list_compared_with_empty_array(array $a): int
{
    // @mago-expect analysis:redundant-comparison,impossible-condition
    if ($a === []) {
        return 0;
    }

    return 1;
}

/**
 * @param non-empty-list<int> $a
 */
function non_empty_list_not_identical_to_empty_array(array $a): int
{
    // @mago-expect analysis:redundant-comparison,redundant-condition
    if ($a !== []) {
        return 1;
    }

    return 0;
}
