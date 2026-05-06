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
array_map([$o, 'doesNotExist'], [1, 2]);
