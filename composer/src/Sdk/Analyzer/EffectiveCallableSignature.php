<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Type\CallableParameter;
use Mago\Sdk\Exception\InvalidArgumentException;

use function array_key_exists;
use function array_key_last;

/**
 * The effective parameters Mago should use when analyzing an invocation.
 * Supplying one also establishes an otherwise-dynamic magic call as known.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 */
final class EffectiveCallableSignature
{
    /**
     * @param list<CallableParameter> $parameters
     */
    public function __construct(
        public readonly array $parameters,
        public readonly bool $allowsNamedArguments = true,
    ) {
        $names = [];
        $optional = false;
        $lastIndex = array_key_last($parameters);
        foreach ($parameters as $index => $parameter) {
            if ($parameter->name !== null) {
                if (array_key_exists($parameter->name, $names)) {
                    throw new InvalidArgumentException(
                        "Effective callable parameter `{$parameter->name}` is duplicated.",
                    );
                }

                $names[$parameter->name] = true;
            }

            if ($parameter->variadic && $index !== $lastIndex) {
                throw new InvalidArgumentException('An effective variadic callable parameter must be last.');
            }
            if ($parameter->variadic && $parameter->hasDefault) {
                throw new InvalidArgumentException('An effective variadic callable parameter cannot have a default.');
            }

            if ($optional && !$parameter->hasDefault && !$parameter->variadic) {
                throw new InvalidArgumentException(
                    'A required effective callable parameter cannot follow an optional one.',
                );
            }

            $optional = $optional || $parameter->hasDefault || $parameter->variadic;
        }
    }
}
