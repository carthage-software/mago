<?php

declare(strict_types=1);

interface InhIUC1
{
    public function alpha(): int;
}

interface InhIUC2 extends InhIUC1
{
    public function beta(): int;
}

interface InhIUC3 extends InhIUC2
{
    public function gamma(): int;
}

class InhIUCImpl implements InhIUC3 {}
