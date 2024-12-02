<?php

foo(1, [
        "user" => "John",
        "email" => "john@example.com",
        "age" => 30,
]);

$loader = new DoctrineChoiceLoader(
    $this->om,
    $this->class,
    $this->idReader,
    $this->objectLoader,
);

test(
    $b = 12 instanceof Foo,
);

foo(
    function (string $bar, string $baz, string $qux): void {
        echo "Hello";
    }
);

class A
{
    #[A\B]
    string $bar;

    public function foo(
        #[A\B]
        #[A\B]
        #[A\B]
        string $bar,
    ) {
    }
}
