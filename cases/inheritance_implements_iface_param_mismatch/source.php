<?php

declare(strict_types=1);

interface InhIfaceParamMismatch
{
    public function feed(string $data): void;
}

class InhIfaceParamMismatchImpl implements InhIfaceParamMismatch
{
    public function feed(int $data): void {}
}
