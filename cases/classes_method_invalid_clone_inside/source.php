<?php

declare(strict_types=1);

final class ClassesCloneInsideInv
{
    public function badClone(): void
    {
        $value = 5;
        /**
         */
        $_ = clone $value;
    }
}
