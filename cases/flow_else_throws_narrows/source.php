<?php

declare(strict_types=1);

/**
 * @throws \RuntimeException
 */
function flow_else_throws_narrows(?int $v): int
{
    if ($v !== null) {
        return $v;
    } else {
        throw new \RuntimeException('null');
    }
}
