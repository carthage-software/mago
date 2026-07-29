<?php

declare(strict_types=1);

class Model
{
    /** @return Builder<static> */
    public static function query(): Builder
    {
        return new Builder(static::class);
    }

    /** @return Builder<static> */
    public static function forwardedQuery(): Builder
    {
        return static::query();
    }
}

/** @template TModel of Model */
class Builder
{
    /** @param class-string<TModel> $model */
    public function __construct(
        public string $model,
    ) {}
}

class ChildModel extends Model
{
}

/** @return Builder<Model> */
function get_items(): Builder
{
    return Model::query();
}

/** @return Builder<ChildModel> */
function get_child_items(): Builder
{
    return ChildModel::query();
}

/** @return Builder<Model> */
function get_literal_items(): Builder
{
    $model = Model::class;

    return $model::query();
}
