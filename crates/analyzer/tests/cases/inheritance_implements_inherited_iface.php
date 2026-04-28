<?php

declare(strict_types=1);

interface InhInherIfaceA
{
    public function alpha(): int;
}

interface InhInherIfaceB extends InhInherIfaceA
{
    public function beta(): int;
}

class InhInherImpl implements InhInherIfaceB
{
    public function alpha(): int
    {
        return 1;
    }

    public function beta(): int
    {
        return 2;
    }
}

(new InhInherImpl())->alpha();
