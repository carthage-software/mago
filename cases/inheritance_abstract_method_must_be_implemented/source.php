<?php

declare(strict_types=1);

abstract class InhAbstractParent
{
    abstract public function action(): void;
}

class InhAbstractMissingChild extends InhAbstractParent {}
