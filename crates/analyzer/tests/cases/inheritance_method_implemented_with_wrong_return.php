<?php

declare(strict_types=1);

abstract class InhWrongRetParent
{
    abstract public function get(): string;
}

class InhWrongRetChild extends InhWrongRetParent
{
    /** @mago-expect analysis:incompatible-return-type */
    public function get(): int
    {
        return 1;
    }
}
