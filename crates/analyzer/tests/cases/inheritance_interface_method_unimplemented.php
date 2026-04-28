<?php

declare(strict_types=1);

interface InhIfaceMethodIface
{
    public function action(): void;
}

/** @mago-expect analysis:unimplemented-abstract-method */
class InhIfaceMethodImpl implements InhIfaceMethodIface
{
}
