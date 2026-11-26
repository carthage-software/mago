<?php

declare(strict_types=1);

class PrivateSetParent
{
    public private(set) int $value = 1;
}

// @mago-expect analysis:override-final-property
class PrivateSetPublicChild extends PrivateSetParent
{
    public int $value = 2;
}

// @mago-expect analysis:override-final-property
class PrivateSetSameChild extends PrivateSetParent
{
    public private(set) int $value = 2;
}

class PrivateSetMiddle extends PrivateSetParent
{
}

// @mago-expect analysis:override-final-property
class PrivateSetGrandchild extends PrivateSetMiddle
{
    public int $value = 2;
}

class ProtectedPrivateSetParent
{
    protected private(set) int $value = 1;
}

// @mago-expect analysis:override-final-property
class ProtectedPrivateSetChild extends ProtectedPrivateSetParent
{
    protected int $value = 2;
}

class PromotedPrivateSetParent
{
    public function __construct(
        public private(set) int $value = 1,
    ) {}
}

// @mago-expect analysis:override-final-property
class PromotedPrivateSetChild extends PromotedPrivateSetParent
{
    public int $value = 2;
}

trait PrivateSetTrait
{
    public private(set) int $value = 1;
}

class TraitPrivateSetParent
{
    use PrivateSetTrait;
}

// @mago-expect analysis:override-final-property
class TraitPrivateSetChild extends TraitPrivateSetParent
{
    public int $value = 2;
}

class ProtectedSetParent
{
    public protected(set) int $value = 1;
}

class ProtectedSetChild extends ProtectedSetParent
{
    public int $value = 2;
}

class PrivatePrivateSetParent
{
    private private(set) int $value = 1;

    public function value(): int
    {
        return $this->value;
    }
}

class PrivatePrivateSetChild extends PrivatePrivateSetParent
{
    public int $value = 2;
}

class ReadonlyParent
{
    public function __construct(
        public readonly int $value = 1,
    ) {}
}

class ReadonlyChild extends ReadonlyParent
{
    public function __construct(
        public readonly int $value = 2,
    ) {}
}

class PrivateSetUntouchedParent
{
    public private(set) int $value = 1;
}

class PrivateSetUntouchedChild extends PrivateSetUntouchedParent
{
    public function read(): int
    {
        return $this->value;
    }
}
