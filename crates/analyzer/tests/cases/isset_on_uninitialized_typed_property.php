<?php

declare(strict_types=1);

/** @mago-expect analysis:missing-constructor */
final class Meta
{
    public string $name;
}

function f(Meta $m): int
{
    $x = isset($m->name) ? 1 : 0;

    return strlen($m->name) + $x;
}

function g(Meta $m): int
{
    if (isset($m->name)) {
        echo 1;
    }

    return strlen($m->name);
}
/** @mago-expect analysis:missing-constructor */
final class Lazy {
    private Lazy $inner;
    public function get(): Lazy {
        if (!isset($this->inner)) {
            $this->inner = new Lazy();
        }
        return $this->inner;
    }
    public function maybe(): ?Lazy {
        if (!isset($this->inner)) {
            return null;
        }
        return $this->inner;
    }
}
/** @param object{bar?: string} $o */
function shape(object $o): string {
    if (!isset($o->bar)) {
        return 'none';
    }
    return $o->bar;
}
