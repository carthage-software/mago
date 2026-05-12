<?php

$a = new Foo;
$b = new Foo();
$c = new Foo(1, 2);
$d = (new Foo)->something();
$e = (new Foo())->something();
$f = (new Foo(1, 2))->something();
$g = (new Foo)->a->b->c;
$h = (new Foo())::CONST;
$i = (new Foo())::method();
$j = (new Foo)?->maybe();
$k = (new Foo)::$static;
$l = foo(new Foo);
$m = foo(new Foo());
$n = foo(new Foo(1, 2));
$o = (new Foo)->method(...);
$p = (new Foo())::method(...);
