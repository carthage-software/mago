<?php

final class Process
{
    private const STATUS_READY = 'ready';

    public function __construct(array $command) {}

    private array $options = [];

    public static function fromShellCommandline(string $command): static
    {
        return new static([$command]);
    }

    public function start(): void {}

    protected function buildCallback(): \Closure
    {
        return static fn() => null;
    }

    private ?int $exitCode = null;

    private function getDescriptors(): array
    {
        return [];
    }

    public function __clone() {}

    public function __destruct() {}
}
