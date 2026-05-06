<?php

declare(strict_types=1);

interface InhNNIface
{
    public function load(): int;
}

class InhNNImpl implements InhNNIface
{
    public function load(): ?int
    {
        return null;
    }
}
