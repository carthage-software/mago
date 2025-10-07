<?php

declare(strict_types=1);

interface IFoo
{
    /**
     * @param callable(string): string $callback
     */
    public function method(int $int, callable $callback): string;

    /**
     * @param int $int
     * @param callable(string): string $callback
     * @return string
     */
    public function methodWithoutTypes($int, $callback);
}

function takesInt(int $int): void {
    echo $int;
}

class Test1 implements IFoo
{
    public function method(int $int, callable $callback): string
    {
        takesInt($int);
        $callback("Hello World!");
        $callback(100); // @mago-expect analysis:invalid-argument
        return "bar";
    }

    public function methodWithoutTypes($int, $callback)
    {
        takesInt($int);
        $callback("Hello World!");
        $callback(100); // @mago-expect analysis:invalid-argument
        return "bar";
    }
}
