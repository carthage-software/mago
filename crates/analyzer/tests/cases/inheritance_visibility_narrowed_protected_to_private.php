<?php

declare(strict_types=1);

abstract class InhVisN3Base
{
    abstract protected function step(): void;
}

class InhVisN3Child extends InhVisN3Base
{
    /** @mago-expect analysis:incompatible-visibility */
    private function step(): void
    {
    }
}
