<?php

declare(strict_types=1);

/**
 * @param array<string, bool>|false $color_support
 */
function unset_key_check_keeps_non_array_parent(array|false $color_support): bool
{
    if (!isset($color_support['text']) || !$color_support['text']) {
        return is_array($color_support);
    }

    return true;
}

function unset_key_check_keeps_mixed_parent(mixed $color_support): bool
{
    if (!isset($color_support['text']) || !$color_support['text']) {
        return is_array($color_support);
    }

    return true;
}
