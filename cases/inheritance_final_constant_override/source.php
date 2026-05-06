<?php

declare(strict_types=1);

class InhFConstParent
{
    final public const int X = 1;
}

class InhFConstChild extends InhFConstParent
{
    public const int X = 2;
}
