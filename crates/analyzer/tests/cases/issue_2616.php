<?php

declare(strict_types=1);

namespace App;

/**
 * @template-covariant T
 * @template-covariant TOptional of bool
 */
class Element
{
    /**
     * @param T $value
     * @param TOptional $optional
     */
    public function __construct(
        public mixed $value,
        public bool $optional = false,
    ) {}
}

/**
 * @template T
 * @template TOptional of bool
 *
 * @param T $value
 * @param TOptional $optional
 *
 * @return Element<T, TOptional>
 */
function element(mixed $value, bool $optional = false): Element
{
    return new Element($value, $optional);
}

// @mago-expect analysis:missing-template-parameter
/** @param Element<string> $e */
function missing_argument(Element $e): void {}

// @mago-expect analysis:excess-template-parameter
/** @param Element<string, true, int> $e */
function excess_argument(Element $e): void {}

// @mago-expect analysis:invalid-template-parameter
/** @param Element<string, int> $e */
function violated_bound(Element $e): void {}

missing_argument(element('a', true));
excess_argument(element('a', true));
// @mago-expect analysis:invalid-argument
violated_bound(element('a', true));

/**
 * @mago-expect analysis:missing-template-parameter
 * @extends Element<string>
 */
final class Reported extends Element {}
