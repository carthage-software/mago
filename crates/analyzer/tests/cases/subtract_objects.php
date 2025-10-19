<?php

interface ExampleInterface {}
class Example implements ExampleInterface {
    public function __construct(public string $name) {}
}

function get_example(string|Example $input): ExampleInterface {
    if ($input instanceof ExampleInterface) {
        return $input;
    }

    return new Example($input);
}
