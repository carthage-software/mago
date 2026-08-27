<?php

declare(strict_types=1);

namespace UninitializedPropertyParentConstructor;

/**
 * Declares a constructor that passes its own argument through to the parent constructor.
 */
final class PassthroughChild extends Base
{
    public function __construct(bool $flag)
    {
        parent::__construct($flag);
    }
}
