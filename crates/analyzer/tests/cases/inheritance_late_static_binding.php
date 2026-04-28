<?php

declare(strict_types=1);

class InhLSBParent
{
    public static function create(): static
    {
        /** @mago-expect analysis:unsafe-instantiation */
        return new static();
    }
}

class InhLSBChild extends InhLSBParent
{
}

$c = InhLSBChild::create();
echo $c::class;
