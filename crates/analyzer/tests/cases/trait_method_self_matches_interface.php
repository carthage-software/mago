<?php

declare(strict_types=1);

interface Duplicable
{
    public function duplicate(): self;

    public function merge(self $other): void;
}

trait Duplicates
{
    public function duplicate(): self
    {
        return new self();
    }

    // @mago-expect analysis:incompatible-parameter-type
    public function merge(self $other): void
    {
        echo spl_object_id($other);
    }
}

final class Concrete implements Duplicable
{
    use Duplicates;
}

abstract class Base
{
    abstract public function make(): ?self;
}

final class Child extends Base
{
    use MakesSelf;
}

trait MakesSelf
{
    public function make(): ?self
    {
        return null;
    }
}
