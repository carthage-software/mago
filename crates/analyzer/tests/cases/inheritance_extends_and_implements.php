<?php

declare(strict_types=1);

interface InhExtImplIface
{
    public function go(): int;
}

abstract class InhExtImplBase
{
    public function helper(): int
    {
        return 1;
    }
}

class InhExtImpl extends InhExtImplBase implements InhExtImplIface
{
    public function go(): int
    {
        return $this->helper();
    }
}

(new InhExtImpl())->go();
