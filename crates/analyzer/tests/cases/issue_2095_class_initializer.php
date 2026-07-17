<?php

declare(strict_types=1);

final readonly class FrameworkManagedComponent
{
    public array $options;

    public function mount(): void
    {
        $this->options = ['some', 'data'];
    }
}

final readonly class AlreadyConstructedFrameworkComponent
{
    public array $options;

    public function __construct()
    {
        $this->options = [];
    }

    public function mount(): void
    {
        // @mago-expect analysis:invalid-property-write
        $this->options = ['some', 'data'];
    }
}
