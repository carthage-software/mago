<?php

declare(strict_types=1);

abstract class InhChainAbsGrand
{
    abstract public function alpha(): int;
}

abstract class InhChainAbsParent extends InhChainAbsGrand
{
    abstract public function beta(): int;
}

/** @mago-expect analysis:unimplemented-abstract-method(2) */
class InhChainAbsChild extends InhChainAbsParent
{
}
