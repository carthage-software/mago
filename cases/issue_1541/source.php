<?php

declare(strict_types=1);

use Deprecated as SomethingElse;

#[Deprecated]
function deprecated_func(): void {}

class Foo
{
    #[\Deprecated]
    public function deprecated_method(): void {}
}

#[Deprecated]
class Bar {}

#[SomethingElse]
const X = 1;

deprecated_func();

(new Foo())->deprecated_method();

new Bar();

echo X;
