<?php

declare(strict_types=1);

class InhMLevelA
{
    public function ping(): string
    {
        return 'A';
    }
}

class InhMLevelB extends InhMLevelA
{
}

class InhMLevelC extends InhMLevelB
{
}

echo (new InhMLevelC())->ping();
