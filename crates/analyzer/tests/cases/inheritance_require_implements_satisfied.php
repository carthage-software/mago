<?php

declare(strict_types=1);

interface InhReqImplOkIface
{
    public function ping(): void;
}

/**
 * @require-implements InhReqImplOkIface
 */
trait InhReqImplOkTrait
{
    public function callPing(): void
    {
        $this->ping();
    }
}

class InhReqImplOk implements InhReqImplOkIface
{
    use InhReqImplOkTrait;

    public function ping(): void
    {
    }
}

(new InhReqImplOk())->callPing();
