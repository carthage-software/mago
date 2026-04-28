<?php

declare(strict_types=1);

class InhParentNoMethod
{
}

class InhChildCallsMissingParent extends InhParentNoMethod
{
    public function go(): void
    {
        /** @mago-expect analysis:non-existent-method */
        parent::missing();
    }
}
