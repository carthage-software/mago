<?php

declare(strict_types=1);

interface InhVarIface
{
    public function call(int ...$values): void;
}

class InhVarImpl implements InhVarIface
{
    public function call(int ...$values): void
    {
    }
}

(new InhVarImpl())->call(1, 2, 3);
