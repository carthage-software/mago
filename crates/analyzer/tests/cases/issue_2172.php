<?php

declare(strict_types=1);

/**
 * @template T of bool
 */
final readonly class EventLoopType
{
    public function __construct(
        /** @var T */
        private bool $core = false,
    ) {}

    /**
     * @return T
     */
    public function isCore(): bool
    {
        return $this->core;
    }

    public function toString(): string
    {
        return match (true) {
            $this->isCore() => 'core',
            default => 'not core',
        };
    }

    public function toInverseString(): string
    {
        return match (false) {
            $this->isCore() => 'not core',
            default => 'core',
        };
    }

    public function compareDirectly(): void
    {
        if ($this->isCore() === true) {
        }

        if (true === $this->isCore()) {
        }
    }
}
