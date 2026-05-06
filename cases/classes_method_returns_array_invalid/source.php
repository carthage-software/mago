<?php

declare(strict_types=1);

final class ClassesMethodArrInv
{
    /**
     * @return list<int>
     */
    public function items(): array
    {
        return ['not', 'ints'];
    }
}
