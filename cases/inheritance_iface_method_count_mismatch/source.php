<?php

declare(strict_types=1);

interface InhIfaceCountIface
{
    public function call(int $a, int $b, int $c): void;
}

class InhIfaceCountImpl implements InhIfaceCountIface
{
    public function call(int $a): void {}
}
