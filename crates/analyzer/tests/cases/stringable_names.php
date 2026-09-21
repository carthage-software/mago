<?php

declare(strict_types=1);

class Foo
{
    public const doFoo = 1;

    public string $doFoo = 'foo';

    public int $doBar = 2;

    public function doFoo(): void {}
}

class StaticFoo
{
    public static string $doFoo = 'foo';

    public static function doBar(): void {}
}

class Stringy implements Stringable
{
    /** @return 'doFoo' */
    public function __toString(): string
    {
        return 'doFoo';
    }
}

class ChildStringy extends Stringy {}

class OtherStringy
{
    /** @return 'doBar' */
    public function __toString(): string
    {
        return 'doBar';
    }
}

function readProperties(Foo $foo): void
{
    $name = new Stringy();

    $foo->{'doFoo'}();
    takeString($foo->{'doFoo'});
    takeString($foo->{new Stringy()});
    takeString($foo->$name);
    takeString($foo->{new ChildStringy()});
    takeOne($foo::{'doFoo'});
    takeString(StaticFoo::${new Stringy()});
    takeString(StaticFoo::$$name);
    takeStringy($name);
}

function readNullableProperty(?Foo $foo): void
{
    takeNullableString($foo?->{new Stringy()});
}

function readVariables(): void
{
    $doFoo = 'test';
    $name = new Stringy();

    takeTest(${'doFoo'});
    takeTest(${new Stringy()});
    takeTest($$name);
    takeTest(${new ChildStringy()});
    takeStringy($name);
}

/** @param Stringy|'doFoo' $name */
function readUnionName(Foo $foo, Stringy|string $name): void
{
    $doFoo = 'test';

    takeString($foo->$name);
    takeString(StaticFoo::$$name);
    takeTest($$name);
}

function readUnionProperty(Foo $foo, Stringy|OtherStringy $name): int|string
{
    return $foo->$name;
}

/**
 * @template T of Stringy
 * @param T $name
 */
function readGenericName(Foo $foo, Stringy $name): void
{
    $doFoo = 'test';

    takeString($foo->$name);
    takeString(StaticFoo::$$name);
    takeTest($$name);
}

/**
 * @template T of 'doFoo'
 * @param T $name
 */
function readGenericStringName(Foo $foo, string $name): string
{
    return $foo->$name;
}

function writeProperties(Foo $foo): void
{
    $foo->{new Stringy()} = 'changed';
    StaticFoo::${new Stringy()} = 'changed';

    // @mago-expect analysis:invalid-property-assignment-value
    $foo->{new Stringy()} = 42;
    // @mago-expect analysis:invalid-property-assignment-value
    StaticFoo::${new Stringy()} = 42;
}

function invalidMethodName(Foo $foo): void
{
    // @mago-expect analysis:invalid-member-selector
    $foo->{new Stringy()}();
}

function invalidStaticMethodName(): void
{
    // @mago-expect analysis:invalid-member-selector
    StaticFoo::{new Stringy()}();
}

function invalidConstantName(Foo $foo): void
{
    // @mago-expect analysis:invalid-constant-selector,impossible-assignment
    $_ = $foo::{new Stringy()};
}

function invalidPropertyName(Foo $foo): void
{
    // @mago-expect analysis:invalid-type-cast,string-member-selector
    $_ = $foo->{new stdClass()};
}

function invalidStaticPropertyName(): void
{
    // @mago-expect analysis:invalid-type-cast
    $_ = StaticFoo::${new stdClass()};
}

function invalidVariableName(): void
{
    // @mago-expect analysis:invalid-type-cast
    $_ = ${new stdClass()};
}

function takeString(string $_): void {}

function takeNullableString(?string $_): void {}

function takeStringy(Stringy $_): void {}

/** @param 'test' $_ */
function takeTest(string $_): void {}

/** @param 1 $_ */
function takeOne(int $_): void {}
