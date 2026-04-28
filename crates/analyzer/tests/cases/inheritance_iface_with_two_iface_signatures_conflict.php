<?php

declare(strict_types=1);

interface InhTwoIfaceA
{
    public function get(): int;
}

interface InhTwoIfaceB
{
    public function get(): string;
}

class InhTwoIfaceImpl implements InhTwoIfaceA, InhTwoIfaceB
{
    /** @mago-expect analysis:incompatible-return-type */
    public function get(): int
    {
        return 1;
    }
}
