<?php

declare(strict_types=1);

/**
 * @template T
 *
 * @mago-expect analysis:unused-template-parameter
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
