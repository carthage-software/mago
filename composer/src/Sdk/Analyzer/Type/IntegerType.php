<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

use Mago\Sdk\Exception\InvalidArgumentException;

/**
 * @api
 * @mago-expect lint:cyclomatic-complexity
 */
final class IntegerType
{
    public function __construct(
        public readonly IntegerTypeKind $kind,
        public readonly ?int $minimum = null,
        public readonly ?int $maximum = null,
    ) {
        $valid = match ($kind) {
            IntegerTypeKind::Literal => $minimum !== null && ($maximum === null || $maximum === $minimum),
            IntegerTypeKind::From => $minimum !== null && $maximum === null,
            IntegerTypeKind::To => $minimum === null && $maximum !== null,
            IntegerTypeKind::Range => $minimum !== null && $maximum !== null && $minimum <= $maximum,
            IntegerTypeKind::General, IntegerTypeKind::UnspecifiedLiteral => $minimum === null && $maximum === null,
        };

        if (!$valid) {
            throw new InvalidArgumentException("Invalid bounds for integer type `{$kind->name}`.");
        }
    }

    public function getLiteralValue(): ?int
    {
        return $this->kind === IntegerTypeKind::Literal ? $this->minimum : null;
    }
}
