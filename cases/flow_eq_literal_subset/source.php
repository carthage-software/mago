<?php

declare(strict_types=1);

/**
 * @param 'foo'|'bar' $tag
 */
function flow_eq_literal_subset(string $tag): bool
{
    return $tag === 'foo';
}
