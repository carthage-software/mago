<?php

declare(strict_types=1);

class InhNoParentClass
{
    public function go(): void
    {
        parent::go();
    }
}
