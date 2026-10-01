<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Assertion\Assertion;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Internal\Analyzer\DefinitionName;

/**
 * Assertions established by one invocation.
 *
 * Map keys are callable parameter names such as `$actual`. Method providers
 * may use `InvocationAssertions::RECEIVER` to target the called object.
 *
 * @api
 */
final class InvocationAssertions
{
    public const RECEIVER = '$this';

    /**
     * @param array<string, list<Assertion>> $assertions
     * @param array<string, list<Assertion>> $ifTrueAssertions
     * @param array<string, list<Assertion>> $ifFalseAssertions
     */
    public function __construct(
        public readonly array $assertions = [],
        public readonly array $ifTrueAssertions = [],
        public readonly array $ifFalseAssertions = [],
    ) {
        self::validate($assertions);
        self::validate($ifTrueAssertions);
        self::validate($ifFalseAssertions);
    }

    /** @internal */
    public function isEmpty(): bool
    {
        return $this->assertions === [] && $this->ifTrueAssertions === [] && $this->ifFalseAssertions === [];
    }

    /** @param array<string, list<Assertion>> $assertions */
    private static function validate(array $assertions): void
    {
        foreach ($assertions as $target => $facts) {
            DefinitionName::assertVariable($target, 'An invocation assertion target');

            if ($facts === []) {
                throw new InvalidArgumentException('Invocation assertion facts must be a non-empty list.');
            }
        }
    }
}
