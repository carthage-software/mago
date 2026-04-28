<?php

declare(strict_types=1);

abstract class InhCannotInstantiateAbs
{
    abstract public function action(): void;
}

/** @mago-expect analysis:abstract-instantiation */
new InhCannotInstantiateAbs();
