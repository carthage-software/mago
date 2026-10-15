<?php

class A
{
    public function fooFoo(): void
    {
    }
}

class B extends A
{
    public function barBar(): void
    {
    }
}

class C
{
    public function bazBaz(): void
    {
    }
}

function getClassNarrows(A|B|C $value): void
{
    switch (get_class($value)) {
        case A::class:
            $value->fooFoo();
            break;

        case B::class:
            $value->barBar();
            break;

        case C::class:
            $value->bazBaz();
            break;
    }
}

function getClassNarrowsViaVariable(A $value): void
{
    $class = get_class($value);
    switch ($class) {
        case B::class:
            $value->barBar();
            break;
    }
}

function getTypeNarrows(int|string $value): void
{
    switch (gettype($value)) {
        case "string":
            $value .= '';
            break;

        case "integer":
            $value += 0;
            break;
    }
}
