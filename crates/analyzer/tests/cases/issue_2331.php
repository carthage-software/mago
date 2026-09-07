<?php

declare(strict_types=1);

namespace Issue2331;

#[\Attribute]
final class Marker {}

final class Concrete {}

/** @template T */
abstract class Container
{
    /** @return list<T> */
    public function elements(): array
    {
        return [];
    }
}

/** @template T */
interface Contract
{
    /** @return T */
    public function value(): object;
}

/** @extends Container<Concrete> */
#[Marker]
final class DocAboveAttribute extends Container {}

#[Marker]
/** @extends Container<Concrete> */
final class DocBetweenAttributeAndClass extends Container {}

#[Marker]
final /** @extends Container<Concrete> */ class DocBetweenModifierAndClass extends Container {}

#[Marker]
/** @implements Contract<Concrete> */
final class ImplementsBetweenAttributeAndClass implements Contract
{
    public function value(): object
    {
        return new Concrete();
    }
}

#[Marker]
/** @extends Contract<Concrete> */
interface ExtendsBetweenAttributeAndInterface extends Contract {}

function docBeforeNew(): Container
{
    return /** @extends Container<Concrete> */ new class() extends Container {};
}

function docBeforeReturn(): Container
{
    /** @extends Container<Concrete> */
    return new class() extends Container {};
}

function docBetweenReturnAndExpression(): Container
{
    return /** @extends Container<Concrete> */ (new class() extends Container {});
}

/** @return list<Container> */
function docBeforeReturnAppliesOnlyToFirstClass(): array
{
    /** @extends Container<Concrete> */
    return [
        new class() extends Container {},
        // @mago-expect analysis:missing-template-parameter
        new class() extends Container {},
    ];
}

function docBeforeReturnDoesNotPassAnArrowFunction(): array
{
    /** @extends Container<Concrete> */
    return [
        static fn(): null => null,
        // @mago-expect analysis:missing-template-parameter
        new class() extends Container {},
    ];
}
