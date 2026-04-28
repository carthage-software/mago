<?php

declare(strict_types=1);

interface InhStaticMUnimpIface
{
    public static function build(): self;
}

/** @mago-expect analysis:unimplemented-abstract-method */
class InhStaticMUnimpImpl implements InhStaticMUnimpIface
{
}
