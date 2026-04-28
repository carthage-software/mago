<?php

declare(strict_types=1);

abstract class InhAbstractParent
{
    abstract public function action(): void;
}

/** @mago-expect analysis:unimplemented-abstract-method */
class InhAbstractMissingChild extends InhAbstractParent
{
}
