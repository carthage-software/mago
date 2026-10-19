<?php

Foo::BAR;
Foo::$property;
Foo::method();
Foo::$method();
Foo::{'BAR'};
Foo::{$method}();
foo(Foo::BAR);
$x = Foo::$property;
