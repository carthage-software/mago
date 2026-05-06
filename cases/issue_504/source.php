<?php

/**
 * @method string getUserById(int $userId)
 */
final class MyClass
{
    public function __call(string $name, array $arguments): string
    {
        return 'hello';
    }
}

$obj = new MyClass();

echo $obj->getUserById(userId: 123);
