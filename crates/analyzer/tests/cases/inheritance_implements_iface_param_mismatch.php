<?php

declare(strict_types=1);

interface InhIfaceParamMismatch
{
    public function feed(string $data): void;
}

class InhIfaceParamMismatchImpl implements InhIfaceParamMismatch
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function feed(int $data): void
    {
    }
}
