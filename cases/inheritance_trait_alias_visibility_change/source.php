<?php

declare(strict_types=1);

trait InhVisAliasTrait
{
    public function open(): string
    {
        return 'open';
    }
}

class InhVisAliasUser
{
    use InhVisAliasTrait {
        open as private hiddenOpen;
    }

    public function expose(): string
    {
        return $this->hiddenOpen();
    }
}

echo (new InhVisAliasUser())->expose();
