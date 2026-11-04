<?php

declare(strict_types=1);

/**
 * @template TFailure of Throwable
 * @param TFailure $failure
 * @return Throwable
 * @throws TFailure
 */
function unresolved_recursive_generic_throw(Throwable $failure): Throwable
{
    return unresolved_recursive_generic_throw($failure);
}
