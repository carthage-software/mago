<?php

declare(strict_types=1);

/** @deprecated */
trait InhDeprTrait
{
}

/** @mago-expect analysis:deprecated-trait */
class InhUsesDeprTrait
{
    use InhDeprTrait;
}
