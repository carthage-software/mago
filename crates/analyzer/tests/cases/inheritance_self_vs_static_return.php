<?php

declare(strict_types=1);

class InhSelfStaticParent
{
    public static function create(): self
    {
        return new self();
    }
}

class InhSelfStaticChild extends InhSelfStaticParent
{
}

$c = InhSelfStaticChild::create();
echo $c::class;
