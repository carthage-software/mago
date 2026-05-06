<?php

declare(strict_types=1);

interface InhOtoRIface
{
    public function call(int $a = 0): void;
}

class InhOtoRImpl implements InhOtoRIface
{
    public function call(int $a): void {}
}
