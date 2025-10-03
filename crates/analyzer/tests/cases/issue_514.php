<?php

declare(strict_types=1);

trait SomeTrait
{
    public function someTraitMethod(): string
    {
        return 'Hello, World!';
    }
}

class SomeClass
{
    use SomeTrait {
        someTraitMethod as someClassMethod;
    }

    public static function callTraitMethod(): string
    {
        return (new self())->someClassMethod();
    }
}

echo strlen(SomeClass::callTraitMethod());

$clx = new SomeClass();

echo strlen($clx->someTraitMethod());
