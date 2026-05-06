<?php

trait TraitWithTypedProperty
{
    public string $name;
}

class UsesTraitNoConstructor
{
    use TraitWithTypedProperty;
}

trait TraitWithDefault
{
    public string $name = 'default';
}

class UsesTraitWithDefault
{
    use TraitWithDefault;
}

trait TraitNeedsInit
{
    public string $value;
}

class InitializesTraitProperty
{
    use TraitNeedsInit;

    public function __construct()
    {
        $this->value = 'initialized';
    }
}

class EmptyConstructorWithTrait
{
    use TraitNeedsInit;

    public string $value;

    public function __construct() {}
}

trait TraitA
{
    public string $a;
}

trait TraitB
{
    public string $b;
}

class UsesMultipleTraitsPartialInit
{
    use TraitA, TraitB;

    public string $b;

    public function __construct()
    {
        $this->a = 'initialized';
    }
}

trait BaseTrait
{
    public string $base;
}

trait ComposedTrait
{
    use BaseTrait;

    public string $composed;
}

class UsesComposedTrait
{
    use ComposedTrait;
}

abstract class AbstractBase
{
    public string $fromParent;
}

trait TraitForChild
{
    public string $fromTrait;
}

class ExtendsAndUsesTrait extends AbstractBase
{
    use TraitForChild;

    public function __construct()
    {
        $this->fromParent = 'parent';
        $this->fromTrait = 'trait';
    }
}

class ExtendsAndUsesTraitNoInit extends AbstractBase
{
    use TraitForChild;
}

trait TraitWithNullable
{
    public ?string $nullable;
}

class UsesNullableTrait
{
    use TraitWithNullable;
}

trait TraitWithStatic
{
    public static string $staticProp = '';
}

class UsesStaticTrait
{
    use TraitWithStatic;
}

trait TraitWithUntyped
{
    public $untyped;
}

class UsesUntypedTrait
{
    use TraitWithUntyped;
}
