<?php

declare(strict_types=1);

namespace UninitializedPropertyParentConstructor;

/**
 * Declares a constructor that supplies the parent constructor argument itself.
 */
final class SpecifyChild extends Base
{
    public function __construct()
    {
        parent::__construct(true);
    }
}
