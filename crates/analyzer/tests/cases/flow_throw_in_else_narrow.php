<?php

declare(strict_types=1);

/**
 * @throws \InvalidArgumentException
 */
function flow_throw_in_else_narrow(null|int $v): int
{
    if ($v !== null) {
        return $v + 1;
    } else {
        throw new \InvalidArgumentException('null');
    }
}
