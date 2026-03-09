<?php

function empty_body(
$a,
$b
) {}

function with_body(
$a,
$b
): void {
echo $a . $b;
}

function single_param(
string $bar,
): void {
echo $bar;
}

function single_param_no_return(
string $bar,
) {
echo $bar;
}

class Foo
{
    public function method(
string $bar,
): void {
echo $bar;
}

    public function method2(
string $a,
string $b,
): void {
echo $a . $b;
}
}

$closure = function (
string $bar,
): void {
echo $bar;
};
