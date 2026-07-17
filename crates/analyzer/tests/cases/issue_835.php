<?php

declare(strict_types=1);

class Test
{
    // @mago-expect analysis:write-only-property
    private readonly \DateTimeInterface $createdAt;
    public readonly string $name;

    public function __construct()
    {
        $this->createdAt = new \DateTimeImmutable();
        $this->name = 'test';
    }

    public function __clone()
    {
        // These should be allowed in PHP 8.3+ (test framework uses 8.4)
        $this->createdAt = new \DateTimeImmutable();
        $this->name = 'cloned';
    }

    public function regularMethod(): void
    {
        // This should still be an error
        $this->name = 'changed'; // @mago-expect analysis:invalid-property-write
    }
}

class Child extends Test
{
    public function __clone()
    {
        // Child class can also re-initialize parent's readonly in __clone
        $this->name = 'child-cloned';
    }
}

final class CloneThroughPrivateHelper
{
    public function __construct(
        public readonly \DateTimeImmutable $date,
    ) {}

    public function __clone()
    {
        $this->reinitializeDate();
    }

    private function reinitializeDate(): void
    {
        $this->date = clone $this->date;
    }
}

final class CloneThroughPublicHelper
{
    public function __construct(
        public readonly \DateTimeImmutable $date,
    ) {}

    public function __clone()
    {
        $this->reinitializeDate();
    }

    public function reinitializeDate(): void
    {
        // The method can also be called outside the clone reinitialization window.
        // @mago-expect analysis:possibly-invalid-property-write
        $this->date = clone $this->date;
    }
}
