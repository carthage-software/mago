<?php

declare(strict_types=1);

interface IntersectionMarker {}

final class IntersectionFoo implements IntersectionMarker {}

final class IntersectionBar implements IntersectionMarker {}

/**
 * @template T
 */
final class IntersectionBox
{
    /** @param T $value */
    public function __construct(
        public mixed $value,
    ) {}
}

/**
 * @type IntersectionObjects = array{foo: IntersectionFoo, bar: IntersectionBar}
 */
interface ParameterVariableIntersectionCombinations
{
    /** @return (IntersectionObjects[$key])&IntersectionMarker */
    public function indexed(string $key): IntersectionMarker;

    /** @return (value-of<$objects>)&IntersectionMarker */
    public function value(array $objects): IntersectionMarker;

    /** @return (new<$class>)&IntersectionMarker */
    public function create(string $class): IntersectionMarker;

    /** @return (template-type<$box, $class, $template>)&IntersectionMarker */
    public function unwrap(IntersectionBox $box, string $class, string $template): IntersectionMarker;

    /** @return (($value is IntersectionFoo ? IntersectionFoo : IntersectionBar))&IntersectionMarker */
    public function conditional(object $value): IntersectionMarker;

    /** @return IntersectionBox<(IntersectionObjects[$key])&IntersectionMarker> */
    public function boxed(string $key): IntersectionBox;

    /** @return callable(): ((IntersectionObjects[$key])&IntersectionMarker) */
    public function callable(string $key): callable;

    /** @return class-string<(IntersectionObjects[$key])&IntersectionMarker> */
    public function className(string $key): string;
}
