<?php

declare(strict_types=1);

class Ancestor
{
    public function lateBinding(): static
    {
        return $this;
    }
}

final class Foo extends Ancestor
{
    public function keep(): static
    {
        return $this;
    }
}

final class Bar extends Ancestor
{
    public function late(): Foo
    {
        return (new Foo())->keep()->lateBinding();
    }
}
