<?php

declare(strict_types=1);

interface InhAbsViaIfaceA
{
    public function a(): void;
}

interface InhAbsViaIfaceB extends InhAbsViaIfaceA
{
    public function b(): void;
}

abstract class InhAbsViaIfaceMid implements InhAbsViaIfaceB
{
    public function a(): void
    {
    }
}

/** @mago-expect analysis:unimplemented-abstract-method */
class InhAbsViaIfaceLeaf extends InhAbsViaIfaceMid
{
}
