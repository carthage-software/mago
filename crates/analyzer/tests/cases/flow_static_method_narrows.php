<?php

declare(strict_types=1);

final class Validator
{
    /**
     * @phpstan-assert-if-true non-empty-string $value
     */
    public static function isNonEmpty(string $value): bool
    {
        return $value !== '';
    }
}

/**
 * @param non-empty-string $v
 */
function take_non_empty(string $v): void
{
    echo $v;
}

function flow_static_method_narrows(string $v): void
{
    if (Validator::isNonEmpty($v)) {
        take_non_empty($v);
    }
}
