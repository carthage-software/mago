<?php

declare(strict_types=1);

namespace Mago\Sdk;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * A source span paired with its logical file name.
 *
 * @api
 */
final class SourceLocation
{
    public function __construct(
        public readonly ?string $file,
        public readonly Span $span,
    ) {
        if ($file === '') {
            throw new InvalidArgumentException('A source location file name cannot be empty.');
        }
    }
}
