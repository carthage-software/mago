<?php

declare(strict_types=1);

/**
 * @template T
 *
 */
final class GenUnusedClassUsedInMethod
{
    /**
     * @template U
     *
     * @param U $value
     *
     * @return U
     */
    public function passthru(mixed $value): mixed
    {
        return $value;
    }
}
