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
$o->helper();
