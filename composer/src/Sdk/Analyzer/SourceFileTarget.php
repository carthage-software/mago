<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

use function str_contains;

/**
 * Selects host files by their forward-slash-separated, workspace-relative path.
 *
 * @api
 */
final class SourceFileTarget
{
    /**
     * @param non-empty-string $pattern
     */
    public function __construct(
        public readonly string $pattern,
    ) {
        if ($pattern === '') {
            throw new InvalidArgumentException('A source-file target pattern cannot be empty.');
        }

        if (str_contains($pattern, "\0")) {
            throw new InvalidArgumentException('A source-file target pattern cannot contain NUL.');
        }
    }
}
