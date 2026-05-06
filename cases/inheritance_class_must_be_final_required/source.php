<?php

declare(strict_types=1);

/** @require-extends InhMustExtendBase */
trait InhMustExtendTrait {}

class InhMustExtendBase {}

class InhMustExtendBad
{
    use InhMustExtendTrait;
}
