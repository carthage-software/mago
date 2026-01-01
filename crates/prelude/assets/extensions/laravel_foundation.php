<?php

use Throwable;

/**
 * @template TException of Throwable
 *
 * @param bool $condition
 * @param TException $exception
 * @param string|null $message
 * @return never
 *
 * @throws TException
 */
function throw_if($condition, $exception, $message = null): void {
    throw new $exception(...func_get_args());
}

/**
 * @template TException of Throwable
 *
 * @param bool $condition
 * @param TException $exception
 * @param string|null $message
 * @return never
 *
 * @throws TException
 */
function throw_unless($condition, $exception, $message = null): void {
    if (!$condition) {
        throw new $exception(...func_get_args());
    }
}
