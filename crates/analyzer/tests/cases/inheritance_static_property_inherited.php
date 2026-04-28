<?php

declare(strict_types=1);

class InhStaticPropParent
{
    public static int $count = 0;
}

class InhStaticPropChild extends InhStaticPropParent
{
}

InhStaticPropChild::$count = 5;
echo InhStaticPropChild::$count;
