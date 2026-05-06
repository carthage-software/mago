<?php

declare(strict_types=1);

abstract class InhWrongRetParent
{
    abstract public function get(): string;
}

class InhWrongRetChild extends InhWrongRetParent
{
    public function get(): int
    {
        return 1;
    }
}
