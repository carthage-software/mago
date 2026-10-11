<?php

$property = new Foo()
    ->bar;

$nullsafeProperty = new Foo()
    ?->bar;

$method = new Foo()
    ->bar();
