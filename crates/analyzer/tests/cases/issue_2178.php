<?php

declare(strict_types=1);

class Model {}

/**
 * @template TRelated of Model
 * @template TDeclaring of Model
 * @template TResult
 */
abstract class Relation
{
    /**
     * @param TRelated $related
     * @param TDeclaring $parent
     */
    public function __construct(
        public Model $related,
        public Model $parent,
    ) {}

    /**
     * @return TRelated
     */
    public function getRelated(): Model
    {
        return $this->related;
    }

    /**
     * @return TResult
     */
    abstract public function getResults(): mixed;

    /**
     * @return $this
     */
    public function where(string $column, mixed $value): static
    {
        return $this;
    }
}

class Builder
{
    /**
     * @param array<array-key, (callable(Relation<mixed, mixed, mixed>): mixed)|string> $relations
     */
    public function with(array $relations): void {}
}

function eagerLoad(Builder $builder): void
{
    $builder->with([
        'templates' => static function (Relation $query): void {
            $query->where('is_guide', false);
        },
    ]);

    $builder->with([
        'templates' =>
            /** @param Relation<mixed, mixed, mixed> $query */
            static function (Relation $query): void {
                $query->where('is_guide', false);
            },
    ]);
}

function getRelatedFromUnspecifiedRelation(Relation $relation): Model
{
    return $relation->getRelated();
}
