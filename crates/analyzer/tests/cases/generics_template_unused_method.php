<?php

declare(strict_types=1);

final class GenUnusedMeth
{
    /**
     * @template T
     *
     * @mago-expect analysis:unused-template-parameter
     */
    public function noop(): void
    {
    }
}
