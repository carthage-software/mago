<?php

declare(strict_types=1);

class InhReqExtSatBase
{
    public function helper(): int
    {
        return 1;
    }
}

/**
 * @require-extends InhReqExtSatBase
 */
trait InhReqExtSatTrait
{
    public function call(): int
    {
        return $this->helper();
    }
}

class InhReqExtSatGood extends InhReqExtSatBase
{
    use InhReqExtSatTrait;
}

echo (new InhReqExtSatGood())->call();
