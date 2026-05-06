<?php

declare(strict_types=1);

trait InhTAbsTrait
{
    abstract public function pendingMethod(): int;
}

class InhTAbsUser
{
    use InhTAbsTrait;
}
