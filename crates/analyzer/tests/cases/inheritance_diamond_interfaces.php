<?php

declare(strict_types=1);

interface InhDiamondTop
{
    public function root(): int;
}

interface InhDiamondLeft extends InhDiamondTop
{
    public function left(): int;
}

interface InhDiamondRight extends InhDiamondTop
{
    public function right(): int;
}

interface InhDiamondBottom extends InhDiamondLeft, InhDiamondRight
{
}

class InhDiamondImpl implements InhDiamondBottom
{
    public function root(): int
    {
        return 0;
    }

    public function left(): int
    {
        return 1;
    }

    public function right(): int
    {
        return 2;
    }
}

(new InhDiamondImpl())->root();
