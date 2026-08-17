<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\Syntax\SourceFile;

/**
 * A deterministic batch of source files selected during codebase scanning.
 *
 * Clear derived state when `firstBatch` is true. Provider requests only begin
 * after the batch for which `lastBatch` is true has returned.
 *
 * @api
 */
final class CodebaseScanContext
{
    /**
     * @param list<SourceFile> $files
     * @internal
     */
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly CancellationTokenInterface $cancellation,
        public readonly array $files,
        public readonly bool $firstBatch,
        public readonly bool $lastBatch,
    ) {}
}
