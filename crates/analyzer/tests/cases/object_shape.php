<?php

class A {
    public int $number;
    public string $str;
    public ?string $optional;
}

class B {
    public int $number;
}

class Nested {
    public int $x;
    public B $b;
}

/**
 * @param object{number: int, str: string, optional: string|null} $obj
 */
function basic_object_test(object $obj): void
{
    echo $obj->number;
    echo $obj->str;
}

/**
 * @param object{x: int, b: B} $nested
 */
function nested_object_test(object $nested): void
{
    echo $nested->b->number;
}

$a = new A();
$b = new B();

basic_object_test($a);

/** @mago-expect analysis:invalid-argument */
basic_object_test($b);

$nested = new Nested();
nested_object_test($nested);

/**
 * @return object{a-b: 1}
 */
function x(): object {
    return (object) ['a-b' => 1];
}

/** @param 1 $a */
function take_one(int $a): void {
  echo $a;
}

take_one(x()->{'a-b'});

/**
 * @return object{'1': 1}
 */
function y(): object {
    $x = (object) [1 => 1];
    return $x;
}

take_one(y()->{'1'});
