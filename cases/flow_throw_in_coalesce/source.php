<?php

declare(strict_types=1);

/**
 * @throws \RuntimeException
 */
function flow_throw_in_coalesce(?string $value): string
{
    return $value ?? throw new \RuntimeException('missing');
}
