<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\PHPVersion;

use function str_contains;

/**
 * Context available while an analyzer plugin initializes.
 *
 * Added stubs exist only in memory. Mago scans them for symbols before parsing
 * the project, but never analyzes, lints, formats, or fixes them as source files.
 *
 * @api
 */
final class InitializationContext
{
    /** @var list<array{non-empty-string, string}> */
    private array $stubs = [];

    /** @var array<string, true> */
    private array $stubNames = [];

    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly CancellationTokenInterface $cancellation,
    ) {}

    public function addStub(string $filename, string $bytes): void
    {
        if ($filename === '' || str_contains($filename, "\0")) {
            throw new InvalidArgumentException('An external stub filename must be non-empty and cannot contain NUL.');
        }

        if ($this->stubNames[$filename] ?? false) {
            throw new InvalidArgumentException("External stub `{$filename}` was added more than once.");
        }

        $this->cancellation->throwIfCancelled();
        $this->stubNames[$filename] = true;
        $this->stubs[] = [$filename, $bytes];
    }

    /** @param array<string, string> $stubs */
    public function addMultipleStubs(array $stubs): void
    {
        foreach ($stubs as $filename => $bytes) {
            $this->addStub($filename, $bytes);
        }
    }

    /**
     * @internal
     *
     * @return list<array{non-empty-string, string}>
     */
    public function getStubs(): array
    {
        return $this->stubs;
    }
}
