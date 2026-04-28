<?php

declare(strict_types=1);

final class Greeter
{
    public function greet(string $name): string
    {
        return 'Hello, ' . $name;
    }
}

$g = new Greeter();
$closure = Closure::fromCallable([$g, 'greet']);
$closure('World');
