<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * Selects named class-like declarations through their ancestry.
 *
 * The ancestor declaration itself never matches.
 *
 * @api
 */
final class ClassLikeTarget
{
    /** @var non-empty-string */
    public readonly string $ancestor;

    private function __construct(string $ancestor)
    {
        if ($ancestor === '') {
            throw new InvalidArgumentException('A class-like analysis target ancestor cannot be empty.');
        }
        DefinitionName::assertSymbol($ancestor, 'A class-like analysis target ancestor');

        $this->ancestor = $ancestor;
    }

    public static function descendantsOf(string $ancestor): self
    {
        return new self($ancestor);
    }
}
