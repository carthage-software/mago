<?php

declare(strict_types=1);

class InhParamParent
{
    public function handle(int $value): void
    {
    }
}

class InhParamChild extends InhParamParent
{
    public function handle(int|string $value): void
    {
    }
}

(new InhParamChild())->handle(1);
(new InhParamChild())->handle('a');
