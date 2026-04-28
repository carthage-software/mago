<?php

declare(strict_types=1);

/**
 * @throws \LogicException
 */
function flow_var_after_throw_branch(int $cond, null|string $v): int
{
    if ($cond > 0) {
        if ($v === null) {
            throw new \LogicException();
        }
    } else {
        return 0;
    }

    return strlen($v);
}
