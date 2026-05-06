<?php

declare(strict_types=1);

abstract class InhCannotInstantiateAbs
{
    abstract public function action(): void;
}

new InhCannotInstantiateAbs();
