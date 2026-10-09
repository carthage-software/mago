<?php

declare(strict_types=1);

namespace GenericDocblockNativeReturn;

/** @template T */
interface Query
{
    /** @return T */
    public function getStuff(): mixed;
}

/** @implements Query<list<string>> */
final class ActualQuery implements Query
{
    public function getStuff(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return ['fred', 1, 2];
    }
}

/** @return list<string> */
function valueComesFromSomewhereElse(): array
{
    return ['foo'];
}

/**
 * @template T
 *
 * @param Query<T> $query
 * @param T $value
 */
function some_function(Query $query, mixed $value): void {}

/** @param list<string> $value */
function takeList(array $value): void {}

some_function(new ActualQuery(), valueComesFromSomewhereElse());
some_function(new ActualQuery(), new ActualQuery()->getStuff());
takeList(new ActualQuery()->getStuff());

/** @implements Query<list<string>> */
final class InheritDocsQuery implements Query
{
    /** @inheritDoc */
    public function getStuff(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return ['fred', 1, 2];
    }
}

some_function(new InheritDocsQuery(), new InheritDocsQuery()->getStuff());
takeList(new InheritDocsQuery()->getStuff());

/** @implements Query<list<string>> */
final class ValidQuery implements Query
{
    public function getStuff(): array
    {
        return ['fred'];
    }
}

some_function(new ValidQuery(), new ValidQuery()->getStuff());
takeList(new ValidQuery()->getStuff());

/** @implements Query<list<string>> */
final class ExplicitQuery implements Query
{
    /** @return non-empty-list<string> */
    public function getStuff(): array
    {
        return ['fred'];
    }
}

/** @param non-empty-list<string> $value */
function takeNonEmptyList(array $value): void {}

takeNonEmptyList(new ExplicitQuery()->getStuff());

/**
 * @template T
 *
 * @implements Query<T>
 */
abstract class BaseQuery implements Query
{
    abstract public function getStuff(): mixed;
}

/** @extends BaseQuery<array{name: string}> */
final class ShapeQuery extends BaseQuery
{
    public function getStuff(): array
    {
        return ['name' => 'fred'];
    }
}

/** @extends BaseQuery<array{name: string}> */
final class InvalidShapeQuery extends BaseQuery
{
    public function getStuff(): array
    {
        // @mago-expect analysis:invalid-return-statement
        return ['name' => 1];
    }
}

/** @param array{name: string} $value */
function takeShape(array $value): void {}

takeShape(new ShapeQuery()->getStuff());

class Value {}

final class SpecificValue extends Value {}

/** @implements Query<Value> */
final class NarrowQuery implements Query
{
    public function getStuff(): SpecificValue
    {
        return new SpecificValue();
    }
}

function takeSpecificValue(SpecificValue $value): void {}

takeSpecificValue(new NarrowQuery()->getStuff());
