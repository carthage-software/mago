<?php

$a = (new Foo())($e);
$a = (new Foo())($e);
$a = (new Foo())($e);
$a = (new Foo(1, 2))($e);
$a = (new Foo())($e)->bar();
$a = (new Foo())(...);
$a = (new Foo())(...);
$a = (new class {
    public function __invoke(): void {}
})();
$a = foo(new Foo());
