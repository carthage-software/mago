<?php

declare(strict_types=1);

namespace UninitializedPropertyParentConstructor;

abstract class Base
{
    protected bool $flag;

    public function __construct(bool $flag)
    {
        $this->flag = $flag;
    }
}
