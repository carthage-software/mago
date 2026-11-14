<?php

declare(strict_types=1);

final class MethodExistsA
{
    public function bar(): void {}
}

final class MethodExistsB
{
    public function baz(): void {}
}

function method_exists_narrows_final_classes(MethodExistsA|MethodExistsB $a): void
{
    if (method_exists($a, 'bar')) {
        $a->bar();

        return;
    }

    $a->baz();
}

function method_exists_keeps_open_classes(object $o): void
{
    if (!method_exists($o, 'bar')) {
        return;
    }

    $o->bar();
}
