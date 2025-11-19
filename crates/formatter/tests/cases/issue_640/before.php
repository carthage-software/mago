<?php

$a = $item instanceof (static::COMPONENT_CLASS);
$b = $item instanceof (Foo::BAR);
$c = $item instanceof (self::CONST);
$d = $item instanceof (Foo::class);
$e = $item instanceof (static::$foo);
$f = $item instanceof (Foo::$bar);
$g = new (static::COMPONENT_CLASS)();
$h = new (Foo::BAR)($arg);
$i = new (self::CONST)();
$j = new (Foo::class)();
$k = new (static::$foo)();
$l = new (Foo::$bar)($arg);
