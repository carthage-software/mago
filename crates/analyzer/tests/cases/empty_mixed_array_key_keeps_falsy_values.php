<?php

declare(strict_types=1);

/**
 * @param array{orderby?: mixed} $query
 */
function empty_mixed_array_key_keeps_falsy_values(array $query): string
{
    if (empty($query['orderby'])) {
        if (isset($query['orderby']) && (is_array($query['orderby']) || false === $query['orderby'])) {
            return 'none';
        }

        return 'default';
    }

    return 'custom';
}

