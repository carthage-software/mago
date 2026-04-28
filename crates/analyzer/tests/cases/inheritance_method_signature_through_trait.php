<?php

declare(strict_types=1);

interface InhTSigIface
{
    public function process(int $x): int;
}

trait InhTSigTrait
{
    public function process(int $x): int
    {
        return $x;
    }
}

class InhTSigImpl implements InhTSigIface
{
    use InhTSigTrait;
}

(new InhTSigImpl())->process(1);
