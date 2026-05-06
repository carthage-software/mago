<?php

declare(strict_types=1);

class InhStaticVsInstParent
{
    public static function build(): int
    {
        return 1;
    }
}

class InhStaticVsInstChild extends InhStaticVsInstParent
{
    public function build(): int
    {
        return 2;
    }
}
