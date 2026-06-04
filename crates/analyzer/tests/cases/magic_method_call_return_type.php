<?php

declare(strict_types=1);

class FluentBuilderMagic
{
    public function foo(): void {}

    /**
     * @param array<array-key, mixed> $_arguments
     */
    public function __call(string $_name, array $_arguments): static
    {
        return $this;
    }
}

function use_magic_call(FluentBuilderMagic $builder): void
{
    /** @mago-expect analysis:non-documented-method */
    $result = $builder->someMethod();

    $result->foo();
}
