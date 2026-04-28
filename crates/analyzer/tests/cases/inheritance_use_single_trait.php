<?php

declare(strict_types=1);

trait InhSingleTrait
{
    public function hello(): string
    {
        return 'hi';
    }
}

class InhSingleTraitUser
{
    use InhSingleTrait;
}

echo (new InhSingleTraitUser())->hello();
