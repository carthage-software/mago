<?php

declare(strict_types=1);

interface InhNullRemoveIface
{
    public function load(): null|string;
}

class InhNullRemoveImpl implements InhNullRemoveIface
{
    public function load(): string
    {
        return 'x';
    }
}
