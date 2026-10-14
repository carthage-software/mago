<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Linter;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Linter\RuleDefinition;

use function array_key_exists;
use function get_debug_type;

/**
 * Resolves the effective option values for one rule invocation.
 *
 * @internal
 */
final class RuleOptions
{
    /**
     * Merges explicit values over the schema defaults.
     *
     * @param array<string, string|int|float|bool|null> $overrides
     *
     * @return array<string, string|int|float|bool|null>
     */
    public static function resolve(RuleDefinition $definition, array $overrides = []): array
    {
        $resolved = [];
        foreach ($definition->options as $option) {
            $resolved[$option->name] = $option->default;
        }

        foreach ($overrides as $name => $value) {
            if (!array_key_exists($name, $resolved)) {
                throw new InvalidArgumentException(
                    "Linter rule `{$definition->code}` has no option named `{$name}`.",
                );
            }

            $default = $resolved[$name];
            if ($default !== null && get_debug_type($value) !== get_debug_type($default)) {
                $expected = get_debug_type($default);
                $actual = get_debug_type($value);

                throw new InvalidArgumentException(
                    "Linter rule `{$definition->code}` option `{$name}` expects {$expected}, got {$actual}.",
                );
            }

            $resolved[$name] = $value;
        }

        return $resolved;
    }
}
