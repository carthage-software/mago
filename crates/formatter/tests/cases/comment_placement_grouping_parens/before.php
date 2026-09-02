<?php

declare(strict_types=1);

/** @param numeric-string $value */
function requires_numeric(string $value): string
{
    return $value;
}

// Mandatory parentheses around pipe arrow functions
$single = '1' |> (/** @param numeric-string $x */ static fn(string $x): string => requires_numeric($x));

// Forced multiline pipe
$chained = '1'
    |> trim(...)
    |> (/** @param numeric-string $x */ static fn(string $x): string => requires_numeric($x))
    |> strrev(...)
    |> strlen(...);

// Multiline docblocks within parentheses
$documented = '1'
    |> (
        /**
         * @param numeric-string $x
         *
         * @return numeric-string
         */
        static fn(string $x): string => requires_numeric($x)
    );

// Line comments within parentheses
$annotated = '1'
    |> (
        // Only ever called with numeric strings.
        static fn(string $x): string => requires_numeric($x)
    );

// Comment outside parentheses
$outside = '1' |> /** not attached to the closure */ (static fn(string $x): string => requires_numeric($x));

// Closure syntax does not require parentheses
$via_closure = '1'
    |> /** @param numeric-string $x */ static function (string $x): string {
        return requires_numeric($x);
    };

// Immediately invoked closure
$eager = (
    /** @return numeric-string */ static function (): string {
        return '1';
    }
)();

// Immediately invoked arrow function
$applied = (/** @param numeric-string $x */ static fn(string $x): string => requires_numeric($x))('1');

// Closure invoked via method
$bound = (
    /** @param numeric-string $x */ function (string $x): string {
        return $this->format($x);
    }
)->call($formatter, '1');

// First-class callable created from a documented closure
$callable = (/** @param numeric-string $x */ static fn(string $x): string => requires_numeric($x))(...);

// Inline `@var` on a class constant used as an instantiation target
$handler = new (/** @var class-string<HandlerInterface> */ self::HANDLER)();

// Inline `@var` on a piped `require` expression
$names = (/** @var list<string> */ require __DIR__ . '/names.php') |> array_values(...);

// Inline `@var` on a piped conditional
$count = (/** @var int $n */ $n > 0 ? $n : 0) |> strval(...);

// Inline `@var` on a pipe used as a unary operand
$magnitude = -(/** @var int $offset */ $offset |> abs(...));

// Inline `@var` on a loop condition assignment
while ((/** @var Row|false $row */ $row = $statement->fetch()) !== false) {
    handle($row);
}
