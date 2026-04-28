<?php

declare(strict_types=1);

final class ClassesMethodArrInv
{
    /**
     * @mago-expect analysis:invalid-return-statement
     * @return list<int>
     */
    public function items(): array
    {
        return ['not', 'ints'];
    }
}
