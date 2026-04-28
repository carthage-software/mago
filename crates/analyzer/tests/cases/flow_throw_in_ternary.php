<?php

declare(strict_types=1);

/**
 * @throws \InvalidArgumentException
 */
function flow_throw_in_ternary(null|string $value): string
{
    return $value !== null ? $value : throw new \InvalidArgumentException('null');
}
