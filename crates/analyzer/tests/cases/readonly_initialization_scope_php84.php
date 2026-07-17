<?php

declare(strict_types=1);

class ReadonlyScopeParentPhp84
{
    public readonly string $value;
}

final class ReadonlyScopeChildPhp84 extends ReadonlyScopeParentPhp84
{
    public function initialize(): void
    {
        // @mago-expect analysis:possibly-invalid-property-write
        $this->value = 'value';
    }
}
