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

/** @mago-expect analysis:unimplemented-abstract-method(2) */
class InhInUnimplImpl implements InhInUnimplB
{
}
