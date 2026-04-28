<?php

declare(strict_types=1);

class InhReqExtBase
{
    public function helper(): int
    {
        return 1;
    }
}

/**
 * @require-extends InhReqExtBase
 */
trait InhReqExtTrait
{
    public function call(): int
    {
        return $this->helper();
    }
}

/** @mago-expect analysis:missing-required-parent */
class InhReqExtMisuse
{
    use InhReqExtTrait;
}
