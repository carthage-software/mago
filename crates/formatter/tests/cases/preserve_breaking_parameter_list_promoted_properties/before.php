<?php

class A {
public function __construct(
private array $a,
) {}
}

class B {
public function __construct(
private array $a,
) {
echo $a;
}
}
