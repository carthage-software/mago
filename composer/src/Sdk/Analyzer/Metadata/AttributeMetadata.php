<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\SourceLocation;

use function in_array;

/**
 * @api
 */
final class AttributeMetadata
{
    /**
     * @param list<AttributeArgumentMetadata> $arguments
     */
    public function __construct(
        public readonly string $name,
        public readonly SourceLocation $location,
        public readonly array $arguments,
    ) {}

    public function getArgument(int $position, string ...$names): ?AttributeArgumentMetadata
    {
        foreach ($this->arguments as $argument) {
            if ($argument->name !== null && in_array($argument->name, $names, true)) {
                return $argument;
            }
        }

        $index = 0;
        foreach ($this->arguments as $argument) {
            if ($argument->name !== null) {
                continue;
            }
            if ($index++ === $position) {
                return $argument;
            }
        }

        return null;
    }
}
