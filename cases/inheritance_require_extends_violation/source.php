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

class InhReqExtMisuse
{
    use InhReqExtTrait;
}
