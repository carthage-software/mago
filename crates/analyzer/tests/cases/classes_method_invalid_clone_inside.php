<?php

declare(strict_types=1);

final class ClassesCloneInsideInv
{
    public function badClone(): void
    {
        $value = 5;
        /**
         * @mago-expect analysis:invalid-clone
         * @mago-expect analysis:impossible-assignment
         */
        $_ = clone $value;
    }
}
