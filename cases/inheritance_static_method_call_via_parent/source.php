<?php

declare(strict_types=1);

class InhStaticParentCall
{
    public static function build(): int
    {
        return 1;
    }
}

class InhStaticParentCallChild extends InhStaticParentCall
{
    public static function compute(): int
    {
        return parent::build() + 1;
    }
}

echo InhStaticParentCallChild::compute();
