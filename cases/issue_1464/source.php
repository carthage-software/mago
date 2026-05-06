<?php

declare(strict_types=1);

class X
{
    public function foo(): void
    {
        $x = new \stdClass();
        if ($x instanceof X) {
            echo 'never';
        }

        if ($x instanceof self) {
            echo 'never';
        }

        if ($x instanceof static) {
            echo 'never';
        }
    }
}
