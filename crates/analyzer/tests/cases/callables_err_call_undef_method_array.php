<?php

declare(strict_types=1);

final class HasOnly
{
    public function exists(): int
    {
        return 1;
    }
}

$o = new HasOnly();
/** @mago-expect analysis:less-specific-nested-argument-type */
array_map([$o, 'noSuchMethod'], [1, 2]);
