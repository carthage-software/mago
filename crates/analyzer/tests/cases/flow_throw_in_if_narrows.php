<?php

declare(strict_types=1);

/**
 * @throws \DomainException
 */
function flow_throw_in_if_narrows(null|string $v): int
{
    if ($v === null) {
        throw new \DomainException('null');
    }

    return strlen($v);
}
