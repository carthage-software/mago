<?php

declare(strict_types=1);

interface InhMatchIfaceA
{
    public function action(): int;
}

interface InhMatchIfaceB
{
    public function action(): int;
}

class InhMatchImpl implements InhMatchIfaceA, InhMatchIfaceB
{
    public function action(): int
    {
        return 1;
    }
}

(new InhMatchImpl())->action();
