<?php

declare(strict_types=1);

interface InhTwoParamIfaceA
{
    public function feed(int $value): void;
}

interface InhTwoParamIfaceB
{
    public function feed(string $value): void;
}

class InhTwoParamImpl implements InhTwoParamIfaceA, InhTwoParamIfaceB
{
    public function feed(int $value): void {}
}
