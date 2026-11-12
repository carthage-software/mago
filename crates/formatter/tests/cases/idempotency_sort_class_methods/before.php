<?php

final class Process
{
    private const STATUS_READY = 'ready';

    public function __destruct()
    {
    }

    private array $options = [];

    private function getDescriptors(): array
    {
        return [];
    }

    public function __clone()
    {
    }

    public static function fromShellCommandline(string $command): static
    {
        return new static([$command]);
    }

    private ?int $exitCode = null;

    public function start(): void
    {
    }

    public function __construct(array $command)
    {
    }

    protected function buildCallback(): \Closure
    {
        return static fn() => null;
    }
}
