<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Receives selected host files while Mago builds the codebase.
 *
 * @api
 */
interface CodebaseScanHook
{
    /**
     * @return non-empty-list<non-empty-string>
     */
    public function getTargets(): array;

    public function scan(CodebaseScanContext $context): void;
}
