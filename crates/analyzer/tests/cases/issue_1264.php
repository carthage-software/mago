<?php

declare(strict_types=1);

/**
 * @template T
 */
interface ResultInterface
{
    /**
     * @template D
     *
     * @param D $default
     *
     * @return T|D
     */
    public function unwrapOr(mixed $default): mixed;
}

/**
 * @return ResultInterface<array<string, string>>
 */
function get_result(): ResultInterface
{
    return get_result();
}

$result = get_result();
$value = $result->unwrapOr([]);
