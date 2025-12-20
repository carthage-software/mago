<?php

declare(strict_types=1);

// @mago-expect analysis:missing-constructor
class Example
{
    public string $value {
        get => $this->value ??= 'default';
    }
}

$e = new Example();
$e->value = 'test';
echo $e->value;

class VirtualExample
{
    private string $internalValue = 'default';

    public string $computed {
        get => $this->internalValue;
    }
}

$v = new VirtualExample();
$v->computed = 'test';  // @mago-expect analysis:invalid-property-write - Virtual property is read-only

// @mago-expect analysis:missing-constructor
class LazyInit
{
    public int $counter {
        get {
            return $this->counter ??= 0;
        }
    }
}

$lazy = new LazyInit();
$lazy->counter = 42;  // Valid - backed property
echo $lazy->counter;
