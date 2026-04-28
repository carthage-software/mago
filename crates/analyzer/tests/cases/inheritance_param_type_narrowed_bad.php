<?php

declare(strict_types=1);

class InhNarrowParent
{
    public function handle(int|string $value): void
    {
    }
}

class InhNarrowChild extends InhNarrowParent
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function handle(int $value): void
    {
    }
}
