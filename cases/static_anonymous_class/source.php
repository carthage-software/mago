<?php

$foo = new class {
    public function bar(): static
    {
        return new static();
    }
};

$foo->bar()->bar()->bar();
