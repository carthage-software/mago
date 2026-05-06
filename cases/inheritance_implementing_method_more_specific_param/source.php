<?php

declare(strict_types=1);

interface InhSpecParamIface
{
    public function feed(mixed $value): void;
}

class InhSpecParamImpl implements InhSpecParamIface
{
    public function feed(int $value): void {}
}
