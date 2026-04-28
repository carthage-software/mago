<?php

declare(strict_types=1);

class InhSelfCallParent
{
    public static function build(): int
    {
        return 1;
    }
}

class InhSelfCallChild extends InhSelfCallParent
{
    public static function go(): int
    {
        return self::build();
    }
}

echo InhSelfCallChild::go();
