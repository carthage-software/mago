<?php

declare(strict_types=1);

class X
{
    public function foo(): void
    {
        $x = new \stdClass();
        if ($x instanceof X) { // @mago-expect analysis:impossible-condition
            echo 'never';
        }

        if ($x instanceof self) { // @mago-expect analysis:impossible-condition
            echo 'never';
        }

        if ($x instanceof static) { // @mago-expect analysis:impossible-condition
            echo 'never';
        }
    }
}
