<?php

declare(strict_types=1);

class InhPMCParent
{
    public function show(): string
    {
        return 'parent';
    }
}

class InhPMCChild extends InhPMCParent
{
    #[\Override]
    public function show(): string
    {
        return 'child:' . parent::show();
    }
}

echo (new InhPMCChild())->show();
