<?php

declare(strict_types=1);

class InhProtParent
{
    protected function helper(): int
    {
        return 1;
    }
}

class InhProtChild extends InhProtParent
{
    public function go(): int
    {
        return $this->helper();
    }
}

echo (new InhProtChild())->go();
