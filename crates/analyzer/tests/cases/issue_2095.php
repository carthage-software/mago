<?php

declare(strict_types=1);

final readonly class PlainDeferredInitialization
{
    public array $options;

    public function mount(): void
    {
        // @mago-expect analysis:possibly-invalid-property-write
        $this->options = ['some', 'data'];
    }
}

final readonly class CoalescedDeferredInitialization
{
    public array $options;

    public function mount(): void
    {
        $this->options ??= ['some', 'data'];
    }
}

final readonly class IssetGuardedDeferredInitialization
{
    public array $options;

    public function mount(): void
    {
        if (!isset($this->options)) {
            $this->options = ['some', 'data'];
        }
    }
}

final readonly class IssetGuardedAfterEarlyReturn
{
    public string $value;

    public function initialize(): void
    {
        if (isset($this->value)) {
            return;
        }

        $this->value = 'value';
    }
}

final readonly class AmbiguousIssetGuard
{
    public string $value;

    public function initialize(bool $force): void
    {
        if (!isset($this->value) || $force) {
            // @mago-expect analysis:possibly-invalid-property-write
            $this->value = 'value';
        }
    }
}

final readonly class NullableCoalescedDeferredInitialization
{
    public ?array $options;

    public function mount(): void
    {
        // @mago-expect analysis:possibly-invalid-property-write
        $this->options ??= ['some', 'data'];
    }
}

final readonly class NullableIssetGuardedDeferredInitialization
{
    public ?array $options;

    public function mount(): void
    {
        if (!isset($this->options)) {
            // @mago-expect analysis:possibly-invalid-property-write
            $this->options = ['some', 'data'];
        }
    }
}

final class RepeatedDeferredInitialization
{
    public readonly string $value;

    public function initialize(): void
    {
        // @mago-expect analysis:possibly-invalid-property-write
        $this->value = 'first';

        // @mago-expect analysis:invalid-property-write
        $this->value = 'second';
    }
}

final class PromotedReadonlyRewrite
{
    public function __construct(
        public readonly string $value,
    ) {
        // @mago-expect analysis:invalid-property-write
        $this->value = 'replacement';
    }
}

final class ReadonlyMutationDuringConstruction
{
    public readonly array $values;

    public function __construct()
    {
        // @mago-expect analysis:invalid-property-write
        $this->values[] = 'value';
    }
}

final class InitializationThroughPrivateHelper
{
    public readonly string $value;

    public function __construct()
    {
        $this->initializeValue();
    }

    private function initializeValue(): void
    {
        $this->value = 'value';
    }
}

final class InitializationThroughPublicHelper
{
    public readonly string $value;

    public function __construct()
    {
        $this->initializeValue();
    }

    public function initializeValue(): void
    {
        // The method can also be called after construction.
        // @mago-expect analysis:possibly-invalid-property-write
        $this->value = 'value';
    }
}
