<?php

declare(strict_types=1);

trait InhTraitWithConst
{
    public const string LABEL = 'hello';

    public function label(): string
    {
        return self::LABEL;
    }
}

class InhTraitConstUser
{
    use InhTraitWithConst;
}

echo (new InhTraitConstUser())->label();
echo InhTraitConstUser::LABEL;
