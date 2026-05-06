<?php

declare(strict_types=1);

interface InhStaticMUnimpIface
{
    public static function build(): self;
}

class InhStaticMUnimpImpl implements InhStaticMUnimpIface {}
