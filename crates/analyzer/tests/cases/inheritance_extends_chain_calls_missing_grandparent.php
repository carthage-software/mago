<?php

declare(strict_types=1);

class InhCallMissingGrand
{
}

class InhCallMissingMid extends InhCallMissingGrand
{
}

class InhCallMissingChild extends InhCallMissingMid
{
    public function go(): void
    {
        /** @mago-expect analysis:non-existent-method */
        parent::missing();
    }
}
