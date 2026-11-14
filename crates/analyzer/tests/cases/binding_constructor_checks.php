<?php

declare(strict_types=1);

class BindingCtorBase
{
    public function __construct(public string $name) {}
}

/** @consistent-constructor */
class BindingCtorChild extends BindingCtorBase
{
    public static function create(): static
    {
        return new static('hello');
    }
}

function binding_ctor_inherited(): void
{
    new BindingCtorChild('direct');
    // @mago-expect analysis:too-few-arguments
    new BindingCtorChild();
}

/** @consistent-constructor */
class BindingCtorConsistent
{
    public function __construct(public string $name) {}

    public static function make(): static
    {
        return new static('base');
    }
}

class BindingCtorMid extends BindingCtorConsistent {}

class BindingCtorGrand extends BindingCtorMid
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function __construct(int $x, int $y)
    {
        parent::__construct((string) ($x + $y));
    }
}

abstract class BindingCtorAbstract
{
    abstract public function __construct(int $a);

    public static function build(): static
    {
        return new static(1);
    }
}

final class BindingCtorWrongType extends BindingCtorAbstract
{
    /** @mago-expect analysis:incompatible-parameter-type */
    public function __construct(string $a) {}
}

final class BindingCtorExtraParam extends BindingCtorAbstract
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function __construct(int $a, int $b) {}
}

interface BindingCtorBuildable
{
    public function __construct();
}

abstract class BindingCtorImplementer implements BindingCtorBuildable
{
    public function __construct() {}

    public static function build(): static
    {
        return new static();
    }
}

final class BindingCtorLeaf extends BindingCtorImplementer
{
    /** @mago-expect analysis:incompatible-parameter-count */
    public function __construct(int $required) {}
}
