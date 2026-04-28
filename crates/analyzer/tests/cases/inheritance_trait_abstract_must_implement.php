<?php

declare(strict_types=1);

trait InhTAbsTrait
{
    abstract public function pendingMethod(): int;
}

/** @mago-expect analysis:unimplemented-abstract-method */
class InhTAbsUser
{
    use InhTAbsTrait;
}
