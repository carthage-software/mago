<?php

abstract class BaseElement {
    abstract public function getName(): string;
}

/** @phpstan-require-extends BaseElement */
trait NamedElement {
    public function getThis(): self
    {
        return $this;
    }

    public function getName(): string
    {
        return 'named';
    }
}

class ConcreteElement extends BaseElement
{
    use NamedElement;
}
