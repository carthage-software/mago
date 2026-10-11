<?php

$x = (new Foo())
    ->bar;

$method = (new Foo())
    ->bar();

$nullsafeProperty = (new Foo())
    ?->bar;

$nullsafeMethod = (new Foo())
    ?->bar();

$chain = (new Foo())
    ->bar->baz();

$inlineProperty = (new Foo())->bar;
$inlineMethod = (new Foo())->bar();

$staticMethod = (new Foo())::bar();
