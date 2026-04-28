<?php

declare(strict_types=1);

trait InhInsteadofA
{
    public function pick(): string
    {
        return 'A';
    }
}

trait InhInsteadofB
{
    public function pick(): string
    {
        return 'B';
    }
}

class InhInsteadofUser
{
    use InhInsteadofA, InhInsteadofB {
        InhInsteadofA::pick insteadof InhInsteadofB;
    }
}

echo (new InhInsteadofUser())->pick();
