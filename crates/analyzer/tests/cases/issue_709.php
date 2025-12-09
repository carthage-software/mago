<?php

declare(strict_types=1);

final class Something
{
    public string $name = '';

    public function someMethod(): void
    {
    }
}

$reflectionClass = new ReflectionClass(Something::class);
$instance = $reflectionClass->newInstanceWithoutConstructor();
$instance->name = 'test';
$instance->someMethod();
