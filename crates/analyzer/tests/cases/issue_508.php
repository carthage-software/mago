<?php

declare(strict_types=1);

interface Y
{
}

abstract class AbstractParent
{
    public function bar(): string
    {
        return 'bar';
    }

    public function __invoke(): string
    {
        return 'foo';
    }
}

class Implementation extends AbstractParent
{
}

class File
{
    public function foo(): string
    {
        $class = new Implementation();

        return $class();
    }

    public function bar(): string
    {
        $class = new Implementation();
        $x = [$class, 'bar'];

        return $x();
    }

    public function baz(): string
    {
        $class = new Implementation();
        $x = [$class, '__invoke'];

        return $x();
    }

    public function qux(Y&AbstractParent $y): string
    {
        $x = [$y, '__invoke'];

        return $x();
    }

    public function duxx(Y&AbstractParent $y): string
    {
        $x = [$y, 'bar'];

        return $x();
    }

    /** @param iterable&AbstractParent $y */
    public function qux2(mixed $y): string
    {
        $x = [$y, '__invoke'];

        return $x();
    }

    /** @param iterable&AbstractParent $y */
    public function duxx2(mixed $y): string
    {
        $x = [$y, 'bar'];

        return $x();
    }

    /** @param iterable&AbstractParent $y */
    public function qux3(mixed $y): string
    {
        $x = [$y, '__invoke'];

        return $x();
    }

    /**
     * @template T of AbstractParent
     *
     * @param T $y
     */
    public function duxx3(object $y): string
    {
        $x = [$y, 'bar'];

        return $x();
    }
}
