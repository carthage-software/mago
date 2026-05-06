<?php

declare(strict_types=1);

/**
 * @method void pseudoMethod(string $s)
 */
class ParentWithCall
{
    public function __call(string $method, array $args): void {}
}

class ChildWithCall extends ParentWithCall
{
    public function pseudoMethod(string $s): void
    {
        parent::pseudoMethod($s);
    }
}

class ParentWithRealMethod
{
    public function realMethod(string $s): void
    {
        echo $s;
    }
}

class ChildCallingParentReal extends ParentWithRealMethod
{
    #[Override]
    public function realMethod(string $s): void
    {
        parent::realMethod($s);
    }
}

class GrandParent
{
    public function inheritedMethod(): void {}
}

class ParentClass extends GrandParent
{
    #[Override]
    public function inheritedMethod(): void
    {
        parent::inheritedMethod();
    }
}

class GrandChild extends ParentClass
{
    #[Override]
    public function inheritedMethod(): void
    {
        parent::inheritedMethod();
    }
}

// Test 4: parent:: with protected method
class ParentWithProtected
{
    protected function protectedMethod(): string
    {
        return 'parent';
    }
}

class ChildWithProtected extends ParentWithProtected
{
    #[Override]
    protected function protectedMethod(): string
    {
        return parent::protectedMethod() . ' child';
    }
}

class ParentWithBoth
{
    public function __call(string $method, array $args): void {}

    public function realMethod(): void {}
}

class ChildCallingBoth extends ParentWithBoth
{
    #[Override]
    public function realMethod(): void
    {
        parent::realMethod();
    }
}

/**
 * @method void pseudoWithoutMagic()
 */
class ParentWithoutCall {}

class ChildCallingWithoutCall extends ParentWithoutCall
{
    public function pseudoWithoutMagic(): void
    {
        parent::pseudoWithoutMagic();
    }
}

/**
 * @method void pseudoWithoutMagic()
 */
final class FinalParentWithoutCall {}

function callFinalPseudo(FinalParentWithoutCall $p): void
{
    $p->pseudoWithoutMagic();
}

class ParentEmpty {}

class ChildCallingNonExistent extends ParentEmpty
{
    public function test(): void
    {
        parent::nonExistentMethod();
    }
}

/**
 * @method static void staticPseudo()
 */
final class WithStaticPseudoOnly
{
    public function __call(string $method, array $args): void {}

    // Missing __callStatic!
}

function testStaticCall(): void
{
    WithStaticPseudoOnly::staticPseudo();
}
