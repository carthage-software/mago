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
array_map([$o, 'noSuchMethod'], [1, 2]);
