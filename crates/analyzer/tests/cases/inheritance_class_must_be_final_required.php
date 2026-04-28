<?php

declare(strict_types=1);

/** @require-extends InhMustExtendBase */
trait InhMustExtendTrait
{
}

class InhMustExtendBase
{
}

/** @mago-expect analysis:missing-required-parent */
class InhMustExtendBad
{
    use InhMustExtendTrait;
}
