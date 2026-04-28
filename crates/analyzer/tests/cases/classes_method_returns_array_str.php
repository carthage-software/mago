<?php

declare(strict_types=1);

final class ClassesArrRetStr
{
    /**
     * @mago-expect analysis:invalid-return-statement
     * @return list<int>
     */
    public function items(): array
    {
        return 'string';
    }
}
