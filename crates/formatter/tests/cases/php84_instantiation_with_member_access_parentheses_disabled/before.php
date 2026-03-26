<?php

$a = (new Foo)->something();
$a = (new Foo())->something();
$a = (new Foo(1, 2))->something();
$a = (new Foo(1, 2))->something()->else()->other()->thing();
$a = new $config['class'];
$a = new $config['class']();
$a = (new Foo)->something(...);
$a = (new Foo())->something(...);
$a = (new Foo(1, 2))->something(...);
$a = (new Foo)::something();
$a = (new Foo())::something();
$a = (new Foo(1, 2))::something();
$a = (new Foo)::something(...);
$a = (new Foo())::something(...);
$a = (new Foo(1, 2))::something(...);
