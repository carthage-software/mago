<?php

declare(strict_types=1);

/**
 * @template T of Throwable
 * @param T $failure
 * @throws T
 */
function throw_from_then_branch(bool $fail, Throwable $failure): int
{
    return $fail ? throw $failure : 1;
}

/**
 * @template T of Throwable
 * @param T $failure
 * @throws T
 */
function throw_from_else_branch(bool $fail, Throwable $failure): int
{
    return $fail ? 1 : throw $failure;
}

/**
 * @template T of Throwable
 * @param T $failure
 * @throws T
 */
function throw_from_finally(bool $fail, Throwable $failure): void
{
    try {
        echo 'work';
    } finally {
        if ($fail) {
            throw $failure;
        }
    }
}
