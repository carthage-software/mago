<?php

declare(strict_types=1);

class InhCVisParent
{
    public const int X = 1;
}

class InhCVisChild extends InhCVisParent
{
    /** @mago-expect analysis:incompatible-constant-access */
    protected const int X = 2;
}
