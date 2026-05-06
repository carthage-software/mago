<?php

declare(strict_types=1);

interface InhConflMidA
{
    public function get(): int;
}

interface InhConflMidB
{
    public function get(): string;
}

interface InhConflChildIface extends InhConflMidA, InhConflMidB {}
