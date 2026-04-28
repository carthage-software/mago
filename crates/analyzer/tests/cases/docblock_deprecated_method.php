<?php

declare(strict_types=1);

final class LegacyBC
{
    /**
     * @deprecated Use newMethod() instead.
     */
    public function old(): int
    {
        return 1;
    }
}

$o = new LegacyBC();
/** @mago-expect analysis:deprecated-method */
echo $o->old();
