<?php

/**
 * Class for testing validation against the @method tag.
 * The __call signature is generic and should not produce errors.
 *
 * @method void specific(int $a, string $b)
 * @method static void staticSpecific(int $a, string $b)
 */
class TestAgainstMethodTag
{
    /** @param array<mixed> $arguments */
    public function __call(string $name, array $arguments): void {}

    /** @param array<mixed> $arguments */
    public static function __callStatic(string $name, array $arguments): void {}
}

/**
 * Class for testing validation against the __call signature.
 * The @method tag is generic, so validation should fall back to __call.
 *
 * @method void generic(mixed ...$args)
 * @method static void staticGeneric(mixed ...$args)
 */
class TestAgainstMagicMethod
{
    /**
     * @param 'foo'|'bar' $name
     * @param array<int, string> $arguments
     */
    public function __call(string $name, array $arguments): void {}

    /**
     * @param 'staticFoo'|'staticBar' $name
     * @param array<int, string> $arguments
     */
    public static function __callStatic(string $name, array $arguments): void {}
}

/**
 * Class for testing validation against both the @method and __call signatures.
 * Both are specific and can conflict.
 *
 * @method void specific(int|string $a)
 * @method static void staticSpecific(int|string $a)
 */
class TestAgainstBoth
{
    /**
     * @param 'specific' $name
     * @param array{0: int} $arguments
     */
    public function __call(string $name, array $arguments): void {}

    /**
     * @param 'staticSpecific' $name
     * @param array{0: int} $arguments
     */
    public static function __callStatic(string $name, array $arguments): void {}
}

function test(): void
{
    $obj = new TestAgainstMethodTag();
    $obj->specific(1, 'hello');
    $obj->specific(1, 'hello', 3);
    $obj->specific(1);
    $obj->specific('hello', 1);

    $obj = new TestAgainstMagicMethod();
    $obj->foo('hello', 'world');
    $obj->bar('another', 'test');
    $obj->baz();
    $obj->foo(123);
    $obj->__call('foo', ['1', '2']);
    $obj->__call('bar', [123]);

    $obj = new TestAgainstBoth();
    $obj->specific(123);
    $obj->specific([]);
    $obj->specific(null);
    $obj->__call('specific', [123]);
    $obj->__call('specific', ['string']);
    $obj->__call('specific', [null]);
}

function testStatic(): void
{
    TestAgainstMethodTag::staticSpecific(1, 'hello');
    TestAgainstMethodTag::staticSpecific(1, 'hello', 3);
    TestAgainstMethodTag::staticSpecific(1);
    TestAgainstMethodTag::staticSpecific('hello', 1);

    TestAgainstMagicMethod::staticFoo('hello', 'world');
    TestAgainstMagicMethod::staticBar('another', 'test');
    TestAgainstMagicMethod::staticBaz();
    TestAgainstMagicMethod::staticFoo(123);
    TestAgainstMagicMethod::__callStatic('staticFoo', ['1', '2']);
    TestAgainstMagicMethod::__callStatic('staticBar', [123]);

    TestAgainstBoth::staticSpecific(123);
    TestAgainstBoth::staticSpecific([]);
    TestAgainstBoth::staticSpecific(null);
    TestAgainstBoth::__callStatic('staticSpecific', [123]);
    TestAgainstBoth::__callStatic('staticSpecific', ['string']);
    TestAgainstBoth::__callStatic('staticSpecific', [null]);
}
