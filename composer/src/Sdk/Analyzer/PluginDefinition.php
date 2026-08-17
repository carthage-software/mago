<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Exception\InvalidArgumentException;

use function array_key_exists;
use function strtolower;

/**
 * Immutable metadata describing an analyzer plugin.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 */
final class PluginDefinition
{
    /** @var non-empty-string */
    public readonly string $identifier;

    /** @var non-empty-string */
    public readonly string $name;

    /** @var non-empty-string */
    public readonly string $description;

    /** @var list<non-empty-string> */
    public readonly array $aliases;

    /**
     * @param string $identifier Globally unique plugin identifier used by analyzer configuration.
     * @param string $name Human-readable plugin name.
     * @param string $description Concise description of the provided analysis behavior.
     * @param list<string> $aliases Alternative analyzer configuration names.
     */
    public function __construct(
        string $identifier,
        string $name,
        string $description,
        array $aliases = [],
        public readonly bool $defaultEnabled = true,
    ) {
        if ($identifier === '' || $name === '' || $description === '') {
            throw new InvalidArgumentException('Analyzer plugin identifier, name, and description cannot be empty.');
        }

        $seen = [];
        $validatedAliases = [];
        foreach ($aliases as $alias) {
            if ($alias === '') {
                throw new InvalidArgumentException("Analyzer plugin `{$identifier}` has an empty alias.");
            }

            $normalized = strtolower($alias);
            if ($normalized === strtolower($identifier) || array_key_exists($normalized, $seen)) {
                throw new InvalidArgumentException("Analyzer plugin `{$identifier}` repeats alias `{$alias}`.");
            }

            $seen[$normalized] = true;
            $validatedAliases[] = $alias;
        }

        $this->identifier = $identifier;
        $this->name = $name;
        $this->description = $description;
        $this->aliases = $validatedAliases;
    }
}
