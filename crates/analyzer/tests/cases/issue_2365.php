<?php

declare(strict_types=1);

namespace Issue2365;

final class Target
{
    /** @param 'type_a'|'type_b' $value */
    public function __construct(string $value) {}
}

/** @param 'type_a' $_ */
function acceptTypeA(string $_): void {}

final class Provider
{
    /** @return lowercase-string */
    public function getDynamicValue(): string
    {
        return 'type_a';
    }
}

final class Reproducer
{
    /** @param 'type_a'|'type_b' $strictValue */
    public function trigger(Provider $provider, string $strictValue): void
    {
        if ($provider->getDynamicValue() !== $strictValue) {
            return;
        }

        new Target($strictValue);
    }

    /** @param 'type_a'|'TYPE_B' $strictValue */
    public function narrow(Provider $provider, string $strictValue): void
    {
        if ($provider->getDynamicValue() !== $strictValue) {
            return;
        }

        acceptTypeA($strictValue);
    }
}
