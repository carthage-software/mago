<?php

declare(strict_types=1);

class InhLSBParent
{
    public static function create(): static
    {
        return new static();
    }
}

class InhLSBChild extends InhLSBParent {}

$c = InhLSBChild::create();
echo $c::class;
