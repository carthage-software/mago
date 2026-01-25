<?php

// OK - Parent initializes property, child calls parent::__construct()
class ParentWithInit
{
    public string $foo;

    public function __construct()
    {
        $this->foo = 'initialized';
    }
}

class ChildCallsParent extends ParentWithInit
{
    public function __construct()
    {
        parent::__construct();
    }
}

class GrandParentWithInit
{
    public string $name;

    public function __construct()
    {
        $this->name = 'grandparent';
    }
}

class MiddleClass extends GrandParentWithInit
{
    public function __construct()
    {
        parent::__construct();
    }
}

class GrandChildCallsParent extends MiddleClass
{
    public function __construct()
    {
        parent::__construct();
    }
}

class ParentInitializesOne
{
    public string $parentProp;

    public function __construct()
    {
        $this->parentProp = 'parent';
    }
}

class ChildAddsOwn extends ParentInitializesOne
{
    public string $childProp;

    public function __construct()
    {
        parent::__construct();
        $this->childProp = 'child';
    }
}

class ParentMultipleProps
{
    public string $a;
    public int $b;
    public bool $c;

    public function __construct()
    {
        $this->a = 'a';
        $this->b = 1;
        $this->c = true;
    }
}

class ChildInheritsMultiple extends ParentMultipleProps
{
    public function __construct()
    {
        parent::__construct();
    }
}

class ParentWithPrivateInit
{
    public string $foo;

    public function __construct()
    {
        $this->init();
    }

    private function init(): void
    {
        $this->foo = 'initialized';
    }
}

class ChildCallsParentWithPrivate extends ParentWithPrivateInit
{
    public function __construct()
    {
        parent::__construct();
    }
}

class ParentWithInitConcrete
{
    public string $foo;

    public function __construct()
    {
        $this->foo = 'initialized';
    }
}

class ChildInitializesOwn extends ParentWithInitConcrete
{
    public string $bar;

    public function __construct()
    {
        parent::__construct();
        $this->bar = 'bar';
    }
}

class ParentOk
{
    public string $parentProp;

    public function __construct()
    {
        $this->parentProp = 'ok';
    }
}

class ChildMissesOwnProp extends ParentOk
{
    // @mago-expect analysis:uninitialized-property
    public string $childProp;

    public function __construct()
    {
        parent::__construct();

        // Missing: $this->childProp = 'something';
    }
}

class ParentPrivateB
{
    private string $_b;

    public function __construct()
    {
        $this->_b = 'parent b';
    }
}

class ChildShadowsPrivate extends ParentPrivateB
{
    // @mago-expect analysis:uninitialized-property
    private string $_b;

    public function __construct()
    {
        parent::__construct();

        // parent::__construct() initializes parent's $_b, not child's $_b
    }
}

class ParentForConditional
{
    public string $prop;

    public function __construct()
    {
        $this->prop = 'initialized';
    }
}

class ChildConditionalParentCall extends ParentForConditional
{
    public function __construct(bool $flag)
    {
        if ($flag) {
            parent::__construct();
        } else {
            parent::__construct();
        }
    }
}

// OK - Abstract parent with constructor
abstract class AbstractParentWithConstructor
{
    public string $foo;

    public function __construct()
    {
        $this->foo = 'abstract parent initialized';
    }
}

class ConcreteChildCallsAbstract extends AbstractParentWithConstructor
{
    public function __construct()
    {
        parent::__construct();
    }
}

class Level1
{
    public string $l1;

    public function __construct()
    {
        $this->l1 = 'level1';
    }
}

class Level2 extends Level1
{
    public string $l2;

    public function __construct()
    {
        parent::__construct();
        $this->l2 = 'level2';
    }
}

class Level3 extends Level2
{
    public string $l3;

    public function __construct()
    {
        parent::__construct();
        $this->l3 = 'level3';
    }
}
