<?php

declare(strict_types=1);

interface InhIfaceI1
{
}

interface InhIfaceI2
{
}

interface InhIntsParamIface
{
    public function feed(InhIfaceI1 $obj): void;
}

class InhIntsParamImpl implements InhIntsParamIface
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function feed(InhIfaceI2 $obj): void
    {
    }
}
