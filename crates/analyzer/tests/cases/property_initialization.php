<?php

// @mago-expect analysis:missing-constructor
class NoConstructor
{
    public int $a;
}

abstract class AbstractParent
{
    public string $foo;
}

// @mago-expect analysis:missing-constructor
class ConcreteChildNoConstructor extends AbstractParent
{
}

// @mago-expect analysis:missing-constructor
class NullableTypedPropertyNoConstructor
{
    private null|bool $_foo;
}

// @mago-expect analysis:missing-constructor
class NullableTypedPropertyNoConstructorWithDocblock
{
    /** @var ?bool */
    private null|bool $_foo;
}

class EmptyConstructor
{
    // @mago-expect analysis:uninitialized-property
    public int $a;

    public function __construct() {}
}

class NotSetInAllBranchesOfIf
{
    // @mago-expect analysis:uninitialized-property
    public int $a;

    public function __construct()
    {
        if (rand(0, 1)) {
            $this->a = 5;
        }
    }
}

class PropertySetInProtectedMethod
{
    // @mago-expect analysis:uninitialized-property
    public int $a;

    public function __construct()
    {
        $this->foo();
    }

    protected function foo(): void
    {
        $this->a = 5;
    }
}

class ProtectedMethodOverrider extends PropertySetInProtectedMethod
{
    protected function foo(): void
    {
    }
}

class PropertySetInPrivateMethodWithIf
{
    // @mago-expect analysis:uninitialized-property
    public int $a;

    public function __construct()
    {
        if (rand(0, 1)) {
            $this->foo();
        }
    }

    private function foo(): void
    {
        $this->a = 5;
    }
}

class ParentWithPrivateB
{
    private string $_b;

    public function __construct()
    {
        $this->_b = 'foo';
    }
}

class ChildWithOwnPrivateB extends ParentWithPrivateB
{
    // @mago-expect analysis:uninitialized-property
    private string $_b;
}

abstract class AbstractWithTypedProperty
{
    // @mago-expect analysis:uninitialized-property
    public string $foo;
}

class ChildEmptyConstructor extends AbstractWithTypedProperty
{
    public function __construct() {}
}

class RecursiveMethodCalls
{
    // @mago-expect analysis:uninitialized-property
    public string $foo;

    public function __construct()
    {
        $this->doThing();
    }

    private function doThing(): void
    {
        if (rand(0, 1)) {
            $this->doOtherThing();
        }
    }

    private function doOtherThing(): void
    {
        if (rand(0, 1)) {
            $this->doThing();
        }
    }
}

abstract class AbstractPrivateConstructor
{
    // @mago-expect analysis:uninitialized-property
    public string $foo;

    private function __construct()
    {
        $this->foo = 'hello';
    }
}

class ChildWithOwnConstructor extends AbstractPrivateConstructor
{
    public function __construct() {}
}

class NullableTypedPropertyEmptyConstructor
{
    // @mago-expect analysis:uninitialized-property
    private null|bool $_foo;

    public function __construct() {}
}

class NullableTypedPropertyEmptyConstructorWithDocblock
{
    /** @var ?bool */
    // @mago-expect analysis:uninitialized-property
    private null|bool $_foo;

    public function __construct() {}
}

class HasConstructor
{
    public string $name;

    public function __construct()
    {
        $this->name = 'test';
    }
}

class HasDefaults
{
    public string $name = 'default';
    public int $age = 0;
}

class PromotedProperties
{
    public function __construct(
        public string $name,
        public int $age,
    ) {}
}

abstract class AbstractClass
{
    public string $name;
}

class NoTypeHint
{
    public $name;
}

class StaticProperties
{
    public static string $name = '';
}

class InitializedViaPrivateMethod
{
    public string $name;

    public function __construct()
    {
        $this->init();
    }

    private function init(): void
    {
        $this->name = 'test';
    }
}

class InitializedViaFinalMethod
{
    public string $name;

    public function __construct()
    {
        $this->init();
    }

    final protected function init(): void
    {
        $this->name = 'test';
    }
}

final class FinalClassProtectedOk
{
    public string $name;

    public function __construct()
    {
        $this->init();
    }

    protected function init(): void
    {
        $this->name = 'test';
    }
}

class TransitiveInitialization
{
    public string $name;

    public function __construct()
    {
        $this->helper();
    }

    private function helper(): void
    {
        $this->doInit();
    }

    private function doInit(): void
    {
        $this->name = 'test';
    }
}

class ConditionalBothBranches
{
    public string $name;

    public function __construct(bool $flag)
    {
        if ($flag) {
            $this->name = 'yes';
        } else {
            $this->name = 'no';
        }
    }
}

abstract class AbstractWithConstructor
{
    public string $foo;

    public function __construct(int $bar)
    {
        $this->foo = (string) $bar;
    }
}

class ConcreteChildImplicit extends AbstractWithConstructor
{
}

class BaseWithConstructor
{
    private string $_aString;

    public function __construct()
    {
        $this->_aString = 'hello';
    }
}

class ExtendsBase extends BaseWithConstructor
{
}

class ConditionalNestedBranches
{
    public string $name;
    public int $age;

    public function __construct(bool $flag1, bool $flag2)
    {
        if ($flag1) {
            if ($flag2) {
                $this->name = 'a';
            } else {
                $this->name = 'b';
            }
            $this->age = 1;
        } else {
            $this->name = 'c';
            $this->age = 2;
        }
    }
}

class PrivateFinalConstructorScenario
{
    public string $foo;

    final private function __construct()
    {
        $this->foo = 'hello';
    }

    public static function create(): self
    {
        return new self();
    }
}
