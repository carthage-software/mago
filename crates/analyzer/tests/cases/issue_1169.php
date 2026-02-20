<?php

class Builder
{
    public function doSomething(): string
    {
        return 'done';
    }
}

/**
 * @mixin Builder
 */
class Relation
{
    private Builder $builder;

    public function __construct()
    {
        $this->builder = new Builder();
    }

    public function __call(string $name, mixed $arguments): mixed
    {
        // @mago-ignore analysis:string-member-selector
        return $this->builder->$name(...$arguments);
    }
}

/**
 * @mixin Builder
 */
trait RelationTrait
{
    private Builder $builder;

    public function __call(string $name, mixed $arguments): mixed
    {
        // @mago-ignore analysis:string-member-selector
        return $this->builder->$name(...$arguments);
    }
}

class HasMany extends Relation {}

class HasManyWithTrait
{
    use RelationTrait;

    public function __construct()
    {
        $this->builder = new Builder();
    }
}

$obj = new HasMany();
$obj->doSomething();

$obj = new HasManyWithTrait();
$obj->doSomething();
