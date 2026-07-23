<?php

declare(strict_types=1);

abstract class Issue2125Dad
{
    public function __construct()
    {
        if (method_exists($this, 'eat')) {
            $this->eat();
        }
    }
}

final class Issue2125Kid extends Issue2125Dad
{
    protected function eat(): void
    {
        echo 'eat';
    }

    // @mago-expect analysis:unused-method
    private function genuinelyUnused(): void {}
}

final class Issue2125PrivateKid extends Issue2125Dad
{
    // @mago-expect analysis:unused-method
    private function eat(): void {}
}

abstract class Issue2125PropertyDad
{
    public function __construct()
    {
        if (property_exists($this, 'read')) {
            if ($this->read === null) {
            }
        }

        if (property_exists($this, 'written')) {
            $this->written = 'written';
        }
    }
}

final class Issue2125PropertyKid extends Issue2125PropertyDad
{
    protected ?string $read = null;

    // @mago-expect analysis:write-only-property
    protected string $written = '';
}

final class Issue2125PrivatePropertyKid extends Issue2125PropertyDad
{
    // @mago-expect analysis:unused-property
    private string $read = '';

    // @mago-expect analysis:unused-property
    private string $written = '';
}

new Issue2125Kid();
new Issue2125PropertyKid();
