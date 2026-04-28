<?php

declare(strict_types=1);

class InhProtOutside
{
    protected function helper(): int
    {
        return 1;
    }
}

$o = new InhProtOutside();
/** @mago-expect analysis:invalid-method-access */
$o->helper();
