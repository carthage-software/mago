<?php

declare(strict_types=1);

class InhConstParent
{
    public const int X = 1;
}

class InhConstChild extends InhConstParent
{
    public const int X = 2;
}

echo InhConstChild::X;
