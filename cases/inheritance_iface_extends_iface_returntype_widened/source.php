<?php

declare(strict_types=1);

interface InhIfaceExtRetParent
{
    public function get(): string;
}

interface InhIfaceExtRetChild extends InhIfaceExtRetParent
{
    public function get(): string|int;
}
