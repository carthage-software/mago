<?php

declare(strict_types=1);

namespace Bar;

use Closure;

class Foo
{
    public function f1(Closure|string $arg): static
    {
        if ($arg instanceof Closure) {
            return $this;
        }

        return $this->f2($arg);
    }

    public function f2(string $_f): static
    {
        return $this;
    }
}
