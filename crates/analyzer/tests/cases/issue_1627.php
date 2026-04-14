<?php

declare(strict_types=1);

namespace MagoBug;

final class NullValue
{
    /**
     * @template T
     *
     * @param NullValue|null|T $value
     *
     * @phpstan-assert-if-true null|NullValue $value
     * @phpstan-assert-if-false T $value
     */
    public static function isNull(mixed $value): bool
    {
        return $value instanceof self || $value === null;
    }
}

final class Input
{
    public ?string $directory = null;
}

function run(?Input $input): string
{
    if (
        $input === null ||
        NullValue::isNull($input->directory) ||
        strlen(trim($input->directory)) === 0
    ) {
        return '<empty>';
    }

    /** @mago-expect analysis:redundant-nullsafe-operator */
    return $input?->directory;
}
