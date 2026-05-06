<?php

declare(strict_types=1);

trait SomeTrait
{
    protected function someTraitMethod(): self
    {
        return $this;
    }
}

class SomeClass
{
    use SomeTrait {
        SomeTrait::someTraitMethod as public someClassMethod;
    }
}

$cls = new SomeClass();
$cls->someClassMethod()->someClassMethod();
