<?php

class A
{
    /**
     * @throws Exception
     */
    public function foo(): void
    {
        throw new Exception();
    }
}

class B
{
    public function foo(): void
    {
        try {
            throw new Exception();
        } catch (Exception) {
        }
    }
}

class C
{
    /**
     */
    public function foo(): void
    {
        throw new Exception();
    }
}

class D
{
    /**
     */
    public function foo(): void
    {
        $a = new A();
        $a->foo();
    }
}
