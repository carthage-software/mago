<?php

declare(strict_types=1);

/**
 * @method void pseudoMethod()
 */
class A {}

class B extends A
{
    public function pseudoMethod(): void
    {
    }
}

class GrandParent
{
    public function realMethod(): void
    {
    }
}

/**
 * @method void realMethod()
 */
class ParentClass extends GrandParent {}

class Child extends ParentClass
{
    #[Override]
    public function realMethod(): void
    {
    }
}

class RealBase
{
    public function concreteMethod(): void
    {
    }
}

class RealChild extends RealBase
{
    #[Override]
    public function concreteMethod(): void
    {
    }
}

/**
 * @method static void staticPseudo()
 */
class WithStaticPseudo {}

class ExtendsStaticPseudo extends WithStaticPseudo
{
    public static function staticPseudo(): void
    {
    }
}
