<?php

declare(strict_types=1);

interface DerivedIntersectionMarker {}

final class DerivedIntersectionFoo implements DerivedIntersectionMarker {}

/** @template T */
final class DerivedIntersectionBox
{
    /** @param T $value */
    public function __construct(
        public mixed $value,
    ) {}
}

interface DerivedTypeIntersectionCombinations
{
    /** @return (array{foo: DerivedIntersectionFoo}['foo'])&DerivedIntersectionMarker */
    public function indexed(): DerivedIntersectionMarker;

    /** @return (value-of<array{foo: DerivedIntersectionFoo}>)&DerivedIntersectionMarker */
    public function value(): DerivedIntersectionMarker;

    /** @return (new<class-string<DerivedIntersectionFoo>>)&DerivedIntersectionMarker */
    public function create(): DerivedIntersectionMarker;

    /** @return (template-type<DerivedIntersectionBox<DerivedIntersectionFoo>, DerivedIntersectionBox::class, 'T'>)&DerivedIntersectionMarker */
    public function unwrap(): DerivedIntersectionMarker;
}
