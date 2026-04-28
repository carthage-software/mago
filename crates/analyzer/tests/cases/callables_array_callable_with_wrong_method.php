<?php

declare(strict_types=1);

final class CalledClass
{
    public function existsMethod(): int
    {
        return 1;
    }
}

$o = new CalledClass();
/** @mago-expect analysis:less-specific-nested-argument-type */
array_map([$o, 'doesNotExist'], [1, 2]);
