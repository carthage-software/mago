<?php

declare(strict_types=1);

trait InhDirectConstTrait
{
    public const int N = 42;
}

class InhDirectConstUser
{
    use InhDirectConstTrait;
}

echo InhDirectConstTrait::N;
