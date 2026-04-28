<?php

declare(strict_types=1);

class InhCtorSigParent
{
    public function __construct(int $a)
    {
    }
}

class InhCtorSigChild extends InhCtorSigParent
{
    public function __construct(string $b)
    {
        parent::__construct(0);
    }
}

new InhCtorSigChild('hello');
