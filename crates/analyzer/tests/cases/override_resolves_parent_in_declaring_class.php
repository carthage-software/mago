<?php

declare(strict_types=1);

class Base {}

class Middle extends Base
{
    public function get(): parent
    {
        return new Base();
    }

    public function set(parent $value): void
    {
    }
}

class Leaf extends Middle
{
    #[\Override]
    public function get(): Base
    {
        return new Base();
    }

    #[\Override]
    public function set(Base $value): void
    {
    }
}

class Unrelated {}

class BadLeaf extends Middle
{
    #[\Override]
    public function get(): Unrelated // @mago-expect analysis:incompatible-return-type
    {
        return new Unrelated();
    }

    #[\Override]
    public function set(Middle $value): void // @mago-expect analysis:incompatible-parameter-type
    {
    }
}

trait ReturnsParent
{
    public function make(): parent
    {
        return new Middle(); // @mago-expect analysis:invalid-return-statement
    }
}

class UsesTrait extends Middle
{
    use ReturnsParent;
}

class OverridesTraitMethod extends UsesTrait
{
    #[\Override]
    public function make(): Middle
    {
        return new Middle();
    }
}
