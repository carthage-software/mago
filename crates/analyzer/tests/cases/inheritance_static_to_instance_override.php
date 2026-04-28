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
    /** @mago-expect analysis:incompatible-static-modifier */
    public function build(): int
    {
        return 2;
    }
}
