<?php

declare(strict_types=1);

interface InhICompatA
{
    public function get(): string;
}

interface InhICompatB extends InhICompatA
{
    /** @mago-expect analysis:incompatible-return-type */
    public function get(): int;
}
