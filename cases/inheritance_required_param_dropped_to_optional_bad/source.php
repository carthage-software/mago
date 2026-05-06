<?php

declare(strict_types=1);

interface InhDropReqIface
{
    public function call(int $a, int $b): void;
}

class InhDropReqImpl implements InhDropReqIface
{
    public function call(int $a): void {}
}
