<?php

declare(strict_types=1);

interface InhRtoOIface
{
    public function call(int $a): void;
}

class InhRtoOImpl implements InhRtoOIface
{
    public function call(int $a = 0): void
    {
    }
}
