<?php

declare(strict_types=1);

namespace Mago\Sdk\Linter;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * One user-configurable option declared by a custom linter rule.
 *
 * Options are scalar so Mago can forward them over the extension protocol
 * without a schema negotiation step. Complex configuration stays on the rule
 * object itself, supplied through its constructor.
 *
 * @api
 */
final class RuleOption
{
    /**
     * @var string|int|float|bool|null
     */
    public readonly string|int|float|bool|null $default;

    /**
     * @param string $name Globally unique option name within its rule.
     * @param string $description Concise option description.
     * @param string|int|float|bool|null $default Value used when Mago supplies no explicit value.
     */
    public function __construct(
        public readonly string $name,
        public readonly string $description,
        string|int|float|bool|null $default = null,
    ) {
        if ($name === '' || $description === '') {
            throw new InvalidArgumentException('A linter rule option name and description cannot be empty.');
        }

        $this->default = $default;
    }
}
