<?php

declare(strict_types=1);

class ReadonlyScopeParentPhp83
{
    public readonly string $value;
}

final class ReadonlyScopeChildPhp83 extends ReadonlyScopeParentPhp83
{
    public function initialize(): void
    {
        // @mago-expect analysis:invalid-property-write
        $this->value = 'value';
    }
}
