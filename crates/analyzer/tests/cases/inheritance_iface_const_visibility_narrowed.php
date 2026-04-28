<?php

declare(strict_types=1);

interface InhIfaceConstVisIface
{
    public const string GREETING = 'hi';
}

class InhIfaceConstVisImpl implements InhIfaceConstVisIface
{
    /** @mago-expect analysis:incompatible-constant-visibility */
    protected const string GREETING = 'bye';
}
