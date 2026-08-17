<?php

declare(strict_types=1);

namespace App;

use Closure;

function acceptObject(object $_): void {}

function acceptClosure(Closure $_): void {}

final readonly class Assert
{
    /** @param Closure(): void $foo */
    public function hello(Closure $foo): void
    {
        $foo->bindTo($this);
    }
}

function testLiterals(object $object): void
{
    $closure = function (): void {};
    $closure->bindTo($object);
    acceptObject($closure);
    acceptClosure($closure);

    $arrow = fn(): null => null;
    $arrow->call($object);

    // @mago-expect analysis:redundant-condition
    if ($closure instanceof Closure) {
    }

    // @mago-expect analysis:redundant-type-comparison,redundant-condition
    if (is_object($arrow)) {
    }
}
