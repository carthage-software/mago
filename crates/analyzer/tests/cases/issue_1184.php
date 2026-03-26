<?php

declare(strict_types=1);

class ParentClass
{
    protected object $_em;

    public function __construct(object $em)
    {
        $this->_em = $em;
    }
}

/**
 * @property object $_em
 */
trait MyTrait
{
    public function getEm(): object
    {
        return $this->_em;
    }
}

class MiddleClass extends ParentClass
{
    use MyTrait;
}

class ChildClass extends MiddleClass
{
    public function __construct(object $em)
    {
        parent::__construct($em);

        $this->doSomething($this->_em);
    }

    public function doSomething(object $em): void {}
}
