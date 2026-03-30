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

/** @mago-expect analysis:deprecated-function */
deprecated_func();

/** @mago-expect analysis:deprecated-method */
(new Foo())->deprecated_method();

/** @mago-expect analysis:deprecated-class */
new Bar();

/** @mago-expect analysis:deprecated-constant */
echo X;
