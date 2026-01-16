<?php

declare(strict_types=1);

interface Foo
{
    /**
     * @param array<string, mixed> $options
     *
     * @return void
     */
    public function buildForm(array $options);
}

abstract class AbstractFoo implements Foo
{
    /**
     * @return void
     */
    public function buildForm(array $options)
    {
    }
}

abstract class MyAbsractFoo extends AbstractFoo {}

class MyFoo extends MyAbsractFoo
{
    public function buildForm(array $options)
    {
    }
}

abstract class MyAbsractBar extends AbstractFoo
{
    /** @inheritDoc */
    public function buildForm(array $options)
    {
    }
}

class MyBar extends MyAbsractBar
{
    /** @inheritDoc */
    public function buildForm(array $options)
    {
    }
}

abstract class MyAbsractBaz extends AbstractFoo
{
    /** @inheritDoc */
    public function buildForm(array $options)
    {
    }
}

class MyBaz extends MyAbsractBaz
{
    public function buildForm(array $options)
    {
    }
}

abstract class MyAbsractQux extends AbstractFoo
{
    public function buildForm(array $options)
    {
    }
}

class MyQux extends MyAbsractQux
{
    public function buildForm(array $options)
    {
    }
}
