<?php

declare(strict_types=1);

/**
 * @template TCases of string|int
 */
abstract class Enumeration {
    /**
     * @return TCases[]
     */
    abstract public static function cases(): array;

    /**
     * @throws InvalidArgumentException
     * @return TCases
     */
    public static function assert(mixed $value): string|int {
        if (!in_array($value, self::cases())) {
            throw new InvalidArgumentException();
        }
        /** @var TCases $value */
        return $value;
    }
}

/**
 * @extends Enumeration<'case1'|'case2'>
 */
class MyEnum extends Enumeration {
    #[\Override]
    public static function cases(): array {
        return ['case1', 'case2'];
    }
}

/**
 * @throws InvalidArgumentException
 * @return 'case1'|'case2'
 */
function enumeration(): string {
    return MyEnum::assert('case2');
}

/** @param 'case1'|'case2' $x */
function use_case(string $x): void {
    echo "case is $x";
}

use_case(enumeration());
