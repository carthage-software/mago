<?php

declare(strict_types=1);

class InhPStaticParent
{
    public static int $count = 0;
}

class InhPStaticChild extends InhPStaticParent
{
    public int $count = 0;
}
