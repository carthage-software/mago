<?php

/** @param int $_ */
function accept_int(int $_): void {}

/** @param string $_ */
function accept_string(string $_): void {}

class Calculator {
    public function add(int $a, int $b): int {
        return $a + $b;
    }

    public function multiply(int $a, int $b, int $c): int {
        return $a * $b * $c;
    }

    public function greet(string $name, string $greeting = "Hello"): string {
        return $greeting . ", " . $name;
    }
}

$calc = new Calculator();

$add_method_fcc = $calc->add(...);
$result1 = $add_method_fcc(5, 3);
accept_int($result1);

$add_five = $calc->add(5, ?);
$result2 = $add_five(3);
accept_int($result2);

$multiply_partial = $calc->multiply(?, 2, ?);
$result4 = $multiply_partial(3, 4);
accept_int($result4);

$greet_saad = $calc->greet(name: ?, greeting: "Hi");
$result5 = $greet_saad("Saad");
accept_string($result5);

class Greeter {
    public function sayHello(string $name): string {
        return "Hello, " . $name;
    }
}

$greeter = new Greeter();
$say_hello_fcc = $greeter->sayHello(...);
$result6 = $say_hello_fcc("World");
accept_string($result6);

$calc->add(?)(1); // @mago-expect analysis:too-few-arguments
$calc->add(?, ?)(1); // @mago-expect analysis:too-few-arguments
$calc->multiply(?, ?, ?)(); // @mago-expect analysis:too-few-arguments
$calc->greet(?)(); // @mago-expect analysis:too-few-arguments
$greeter->sayHello(?)(123); // @mago-expect analysis:invalid-argument
