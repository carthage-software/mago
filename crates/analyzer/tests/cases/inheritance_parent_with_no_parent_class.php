<?php

declare(strict_types=1);

class InhNoParentClass
{
    public function go(): void
    {
        /** @mago-expect analysis:invalid-parent-type */
        parent::go();
    }
}
