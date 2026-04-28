<?php

declare(strict_types=1);

/**
 * @throws \InvalidArgumentException
 */
function flow_throw_then_use(null|string $v): int
{
    if ($v === null) {
        throw new \InvalidArgumentException('null');
    }

    return strlen($v);
}
