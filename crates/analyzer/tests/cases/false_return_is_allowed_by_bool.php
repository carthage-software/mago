<?php

declare(strict_types=1);

function false_return_is_allowed_by_bool(string $haystack): bool|int
{
    return strpos($haystack, 'a');
}

function false_return_is_not_allowed_by_true(string $haystack): true|int
{
    /** @mago-expect analysis:falsable-return-statement,invalid-return-statement */
    return strpos($haystack, 'a');
}
