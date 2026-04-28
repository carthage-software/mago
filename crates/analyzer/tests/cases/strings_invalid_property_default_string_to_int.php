<?php

declare(strict_types=1);

final class Foo
{
    /** @mago-expect analysis:invalid-property-default-value */
    public int $x = 'hello';
}

new Foo();
