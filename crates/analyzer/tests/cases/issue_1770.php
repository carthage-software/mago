<?php

declare(strict_types=1);

namespace App;

use Closure;

class Test
{
    private Closure $callback;

    public function setCallback(mixed $callback): void
    {
        if ($callback instanceof Closure) {
            $this->callback = $callback;
        }
    }

    public function setCallbackWithTypeHint(Closure $callback): void
    {
        $this->callback = $callback;
    }

    public function getClosure(): Closure
    {
        return $this->callback;
    }
}
