<?php

declare(strict_types=1);

/** @deprecated */
trait InhDeprTrait {}

class InhUsesDeprTrait
{
    use InhDeprTrait;
}
