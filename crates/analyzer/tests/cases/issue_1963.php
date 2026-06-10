<?php

declare(strict_types=1);

namespace MyNamespace;

/**
 * @psalm-consistent-constructor
 *
 * @method static static magicMethod()
 */
class MyClass
{
    /**
     * @param array<array-key, mixed> $_args
     */
    public static function __callStatic(string $_name, array $_args): static
    {
        return new static();
    }

    public function foo(): string
    {
        return 'hello';
    }
}

echo MyClass::magicMethod()->foo();
