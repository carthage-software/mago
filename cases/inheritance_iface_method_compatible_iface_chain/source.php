<?php

declare(strict_types=1);

interface InhICompatA
{
    public function get(): string;
}

interface InhICompatB extends InhICompatA
{
    public function get(): int;
}
