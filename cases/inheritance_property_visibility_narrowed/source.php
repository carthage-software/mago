<?php

declare(strict_types=1);

class InhPVisParent
{
    public int $value = 0;
}

class InhPVisChild extends InhPVisParent
{
    protected int $value = 0;
}
