<?php

declare(strict_types=1);

interface InhIfaceExtRetParent
{
    public function get(): string;
}

interface InhIfaceExtRetChild extends InhIfaceExtRetParent
{
    /** @mago-expect analysis:incompatible-return-type */
    public function get(): string|int;
}
