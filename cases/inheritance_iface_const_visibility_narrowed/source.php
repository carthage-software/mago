<?php

declare(strict_types=1);

interface InhIfaceConstVisIface
{
    public const string GREETING = 'hi';
}

class InhIfaceConstVisImpl implements InhIfaceConstVisIface
{
    protected const string GREETING = 'bye';
}
