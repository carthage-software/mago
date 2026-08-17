<?php

declare(strict_types=1);

namespace Mago\Sdk\Linter;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Syntax\NodeKind;

use function array_key_exists;

/**
 * Immutable metadata describing a custom linter rule.
 *
 * @api
 */
final class RuleDefinition
{
    /**
     * @var non-empty-string
     */
    public readonly string $code;

    /**
     * @var non-empty-string
     */
    public readonly string $name;

    /**
     * @var non-empty-string
     */
    public readonly string $description;

    /**
     * @var non-empty-list<NodeKind>
     */
    public readonly array $targets;

    /**
     * @param string $code Globally unique issue code.
     * @param string $name Human-readable rule name.
     * @param string $description Concise rule description.
     * @param list<NodeKind> $targets
     * @mago-expect lint:excessive-parameter-list
     */
    public function __construct(
        string $code,
        string $name,
        string $description,
        public readonly Level $defaultLevel,
        public readonly bool $defaultEnabled,
        array $targets,
    ) {
        if ($code === '' || $name === '' || $description === '') {
            throw new InvalidArgumentException('Rule code, name, and description cannot be empty.');
        }

        if ($targets === []) {
            throw new InvalidArgumentException("Linter rule `{$code}` must subscribe to at least one node kind.");
        }

        $seen = [];
        foreach ($targets as $target) {
            if (array_key_exists($target->value, $seen)) {
                throw new InvalidArgumentException(
                    "Linter rule `{$code}` subscribes to `{$target->value}` more than once.",
                );
            }

            $seen[$target->value] = true;
        }

        $this->code = $code;
        $this->name = $name;
        $this->description = $description;
        $this->targets = $targets;
    }
}
