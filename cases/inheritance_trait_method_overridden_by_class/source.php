<?php

declare(strict_types=1);

trait InhTraitOverridden
{
    public function pick(): string
    {
        return 'trait';
    }
}

class InhTraitOverridenUser
{
    use InhTraitOverridden;

    public function pick(): string
    {
        return 'class';
    }
}

echo (new InhTraitOverridenUser())->pick();
