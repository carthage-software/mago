<?php

declare(strict_types=1);

interface InhReqImplIface
{
    public function ping(): void;
}

/**
 * @require-implements InhReqImplIface
 */
trait InhReqImplTrait
{
    public function callPing(): void
    {
        $this->ping();
    }
}

/** @mago-expect analysis:missing-required-interface */
class InhReqImplBad
{
    use InhReqImplTrait;

    public function ping(): void
    {
    }
}
