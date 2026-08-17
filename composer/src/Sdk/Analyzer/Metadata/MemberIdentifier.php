<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * Identifies one class-like member for batched metadata queries.
 *
 * @api
 */
final class MemberIdentifier
{
    public function __construct(
        public readonly string $class,
        public readonly string $member,
    ) {
        if ($class === '' || $member === '') {
            throw new InvalidArgumentException('A member identifier requires non-empty class and member names.');
        }
    }
}
