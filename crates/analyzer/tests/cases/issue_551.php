<?php declare(strict_types=1);

class X {
  final public function __construct() { }
    static function create(): static { return new static(); }
}

class Y extends X { }

function f(Y $y): Y { return $y; }

f(Y::create());
f(Y::create(...)());
