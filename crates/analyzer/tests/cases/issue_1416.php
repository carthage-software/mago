<?php

declare(strict_types=1);

class Foo1416
{
    public string $a = '';
    public string $b = '';
    public array $aa = [];
    public array $bb = [];

    function bar(): array
    {
        $result = [];
        foreach (['a', 'b'] as $name) {
            if ($this->{$name}) {
                $result[] = $name;
            }
        }
        foreach (['aa', 'bb'] as $name) {
            if (is_array($this->{$name}) && count($this->{$name}) > 0) {
                $result[] = $name;
            }
        }
        return $result;
    }
}
