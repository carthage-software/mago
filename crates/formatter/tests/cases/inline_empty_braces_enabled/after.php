<?php

if ($condition) {}

function empty_function() {}

$closure = function () {};

class EmptyClass {}

trait EmptyTrait {}

interface EmptyInterface {}

enum EmptyEnum {}

class Example
{
    public function __construct() {}

    public function emptyMethod() {}
}

$anon = new class {};
