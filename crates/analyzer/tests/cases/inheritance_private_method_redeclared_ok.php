<?php

declare(strict_types=1);

class InhPrivParent
{
    private function helper(): int
    {
        return 1;
    }

    public function show(): int
    {
        return $this->helper();
    }
}

class InhPrivChild extends InhPrivParent
{
    private function helper(): string
    {
        return 'x';
    }

    public function showChild(): string
    {
        return $this->helper();
    }
}

echo (new InhPrivChild())->show();
echo (new InhPrivChild())->showChild();
