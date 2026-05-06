<?php

declare(strict_types=1);

interface InhTraitBadIface
{
    public function process(int $x): int;
}

trait InhTraitBadTrait
{
    public function process(string $x): int
    {
        return strlen($x);
    }
}

class InhTraitBadImpl implements InhTraitBadIface
{
    use InhTraitBadTrait;
}
