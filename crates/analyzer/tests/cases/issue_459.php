<?php

class A
{
    public function getFoo(): string
    {
        return 'foo';
    }
}

class B
{
    public function getBar(): string
    {
        return 'bar';
    }
}

class C extends A
{
}

class D extends B
{
    public function getBaz(): string
    {
        return 'baz';
    }
}

/**
 * @psalm-assert-if-true A $obj
 */
function is_instance_of_a(object $obj): bool
{
    return $obj instanceof A;
}

/**
 * @psalm-assert-if-true B $obj
 */
function is_instance_of_b(object $obj): bool
{
    return $obj instanceof B;
}

/**
 * @psalm-assert-if-true D $obj
 */
function is_instance_of_d(object $obj): bool
{
    return $obj instanceof D;
}

function process(A|B $obj): string
{
    return match (true) {
        $obj instanceof A => $obj->getFoo(),
        $obj instanceof B => match (true) {
            $obj instanceof D => $obj->getBaz(),
            default => $obj->getBar(),
        },
    };
}

function process_assertions(A|B $obj): string
{
    return match (true) {
        is_instance_of_a($obj) => $obj->getFoo(),
        is_instance_of_b($obj) => match (true) {
            is_instance_of_d($obj) => $obj->getBaz(),
            default => $obj->getBar(),
        },
    };
}

function booleans_arr(bool $a, bool $b): string
{
    return match ([$a, $b]) {
        [true, true] => 'a and b',
        [true, false] => 'a and not b',
        [false, true] => 'not a and b',
        [false, false] => 'not a and not b',
    };
}

function booleans3_arr(bool $a, bool $b, bool $c): string
{
    return match ([$a, $b, $c]) {
        [true, true, true] => 'a and b and c',
        [true, true, false] => 'a and b and not c',
        [true, false, true] => 'a and not b and c',
        [true, false, false] => 'a and not b and not c',
        [false, true, true] => 'not a and b and c',
        [false, true, false] => 'not a and b and not c',
        [false, false, true] => 'not a and not b and c',
        [false, false, false] => 'not a and not b and not c',
    };
}
