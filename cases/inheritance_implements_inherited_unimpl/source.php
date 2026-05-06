<?php

declare(strict_types=1);

interface InhInUnimplA
{
    public function alpha(): int;
}

interface InhInUnimplB extends InhInUnimplA
{
    public function beta(): int;
}

class InhInUnimplImpl implements InhInUnimplB {}
