<?php

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

class Level4 extends Level3
{
    public string $l4;

    public function __construct()
    {
        parent::__construct();
        $this->l4 = 'level4';
    }
}

class Level5 extends Level4
{
    public string $l5;

    public function __construct()
    {
        parent::__construct();
        $this->l5 = 'level5';
    }
}

class DeepLevel1
{
    public string $d1;

    public function __construct()
    {
        $this->d1 = 'deep1';
    }
}

class DeepLevel2 extends DeepLevel1
{
    public string $d2;

    public function __construct()
    {
        parent::__construct();
        $this->d2 = 'deep2';
    }
}

class DeepLevel3 extends DeepLevel2
{
    public string $d3;

    public function __construct()
    {
        $this->d3 = 'deep3';
    }
}

abstract class AbstractL1
{
    public string $a1;
}

abstract class AbstractL2 extends AbstractL1
{
    public string $a2;
}

abstract class AbstractL3 extends AbstractL2
{
    public string $a3;
}

class ConcreteAtEnd extends AbstractL3
{
    public function __construct()
    {
        $this->a1 = 'a1';
        $this->a2 = 'a2';
        $this->a3 = 'a3';
    }
}

// @mago-expect analysis:missing-constructor
class ConcreteAtEndNoInit extends AbstractL3
{
}

class Shadow1
{
    private string $_shadow;

    public function __construct()
    {
        $this->_shadow = 's1';
    }
}

class Shadow2 extends Shadow1
{
    private string $_shadow;

    public function __construct()
    {
        parent::__construct();
        $this->_shadow = 's2';
    }
}

class Shadow3 extends Shadow2
{
    // @mago-expect analysis:uninitialized-property
    private string $_shadow;

    public function __construct()
    {
        parent::__construct();

        // parent initializes Shadow2::$_shadow, not Shadow3::$_shadow
    }
}

class A1
{
    public string $p1;

    public function __construct()
    {
        $this->p1 = 'a1';
    }
}

class A2 extends A1
{
    public string $p2;

    public function __construct()
    {
        parent::__construct();
        $this->p2 = 'a2';
    }
}

class A3 extends A2
{
    public string $p3;

    public function __construct()
    {
        parent::__construct();
        $this->p3 = 'a3';
    }
}

class A4 extends A3
{
    public string $p4;

    public function __construct()
    {
        parent::__construct();
        $this->p4 = 'a4';
    }
}

class A5 extends A4
{
    public string $p5;

    public function __construct()
    {
        parent::__construct();
        $this->p5 = 'a5';
    }
}

class A6 extends A5
{
    public string $p6;

    public function __construct()
    {
        parent::__construct();
        $this->p6 = 'a6';
    }
}

trait DeepTrait
{
    public string $traitProp;
}

abstract class DeepBase1
{
    public string $base1;
}

abstract class DeepBase2 extends DeepBase1
{
    public string $base2;
}

class DeepLeafWithTrait extends DeepBase2
{
    use DeepTrait;

    public function __construct()
    {
        $this->base1 = 'b1';
        $this->base2 = 'b2';
        $this->traitProp = 'trait';
    }
}

// @mago-expect analysis:missing-constructor
class DeepLeafWithTraitNoInit extends DeepBase2
{
    use DeepTrait;
}
