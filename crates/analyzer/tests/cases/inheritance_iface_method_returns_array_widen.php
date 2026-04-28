<?php

declare(strict_types=1);

interface InhRetArrIface
{
    /** @return list<int> */
    public function items(): array;
}

class InhRetArrImpl implements InhRetArrIface
{
    /**
     * @mago-expect analysis:invalid-return-statement
     *
     * @return list<int>
     */
    public function items(): array
    {
        /** @var array<int> $a */
        $a = [];
        return $a;
    }
}
