<?php

declare(strict_types=1);

class QueryBuilder
{
    final public function __construct() {}

    /**
     * @return $this
     */
    public function lock(): static
    {
        return $this;
    }

    /**
     * @return static
     */
    public function fresh(): static
    {
        return new static();
    }

    /**
     * @return static
     */
    public static function make(): static
    {
        return new static();
    }

    /**
     * @param static $other
     *
     * @return static
     */
    public function merge(self $other): static
    {
        return $this;
    }
}

/**
 * @mixin QueryBuilder
 */
class EloquentBuilder
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }

    public static function __callStatic(string $name, array $arguments): mixed
    {
        return new static();
    }
}

function callThisReturningMixinMethod(EloquentBuilder $builder): EloquentBuilder
{
    return $builder->lock();
}

function callStaticReturningMixinMethod(EloquentBuilder $builder): EloquentBuilder
{
    return $builder->fresh();
}

function callStaticMixinMethod(): QueryBuilder
{
    return EloquentBuilder::make();
}

function passReceiverToStaticParameterOfMixinMethod(EloquentBuilder $builder, EloquentBuilder $other): EloquentBuilder
{
    // @mago-expect analysis:less-specific-argument - `static` parameters bind to the receiver (`EloquentBuilder&static`)
    return $builder->merge($other);
}

/**
 * @template TModel of object
 */
final class TemplatedBuilder
{
    /** @var TModel */
    private object $model;

    /**
     * @param TModel $model
     */
    final public function __construct(object $model)
    {
        $this->model = $model;
    }

    /**
     * @return $this
     */
    public function lockTemplated(): static
    {
        return $this;
    }

    /**
     * @return static
     */
    public function freshTemplated(): static
    {
        return new static($this->model);
    }

    /**
     * @return TModel
     */
    public function model(): object
    {
        return $this->model;
    }
}

final class User {}

/**
 * @mixin TemplatedBuilder<User>
 */
final class UserBuilder
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningTemplatedMixinMethod(UserBuilder $builder): UserBuilder
{
    return $builder->lockTemplated();
}

function callStaticReturningTemplatedMixinMethod(UserBuilder $builder): UserBuilder
{
    return $builder->freshTemplated();
}

function callClassTemplateReturningMixinMethod(UserBuilder $builder): User
{
    return $builder->model();
}

final class Widget {}

/**
 * @template TItem of object
 * @mixin TemplatedBuilder<User>
 */
final class WidgetBuilder
{
    /** @var TItem */
    private object $item;

    /**
     * @param TItem $item
     */
    final public function __construct(object $item)
    {
        $this->item = $item;
    }

    /**
     * @return TItem
     */
    public function item(): object
    {
        return $this->item;
    }

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

/**
 * @param WidgetBuilder<Widget> $builder
 *
 * @return WidgetBuilder<Widget>
 */
function callStaticReturningMixinMethodOnTemplatedReceiver(WidgetBuilder $builder): WidgetBuilder
{
    return $builder->freshTemplated();
}

/**
 * @param WidgetBuilder<Widget> $builder
 */
function callClassTemplateReturningMixinMethodOnTemplatedReceiver(WidgetBuilder $builder): User
{
    return $builder->model();
}

/**
 * @template TItem of object
 * @mixin TemplatedBuilder<TItem>
 */
final class ForwardingBuilder
{
    /** @var TItem */
    private object $item;

    /**
     * @param TItem $item
     */
    final public function __construct(object $item)
    {
        $this->item = $item;
    }

    /**
     * @return TItem
     */
    public function item(): object
    {
        return $this->item;
    }

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

/**
 * @param ForwardingBuilder<Widget> $builder
 */
function callClassTemplateReturningForwardedMixinMethod(ForwardingBuilder $builder): Widget
{
    return $builder->model();
}

/**
 * @template TDelegate of QueryBuilder
 * @mixin TDelegate
 */
final class DelegatingBuilder
{
    /** @var TDelegate */
    private QueryBuilder $delegate;

    /**
     * @param TDelegate $delegate
     */
    final public function __construct(QueryBuilder $delegate)
    {
        $this->delegate = $delegate;
    }

    /**
     * @return TDelegate
     */
    public function delegate(): QueryBuilder
    {
        return $this->delegate;
    }

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningGenericParameterMixinMethod(DelegatingBuilder $builder): DelegatingBuilder
{
    return $builder->lock();
}

function callStaticReturningGenericParameterMixinMethod(DelegatingBuilder $builder): DelegatingBuilder
{
    return $builder->fresh();
}

final class ChildEloquentBuilder extends EloquentBuilder {}

function callThisReturningMixinMethodOnSubclass(ChildEloquentBuilder $builder): ChildEloquentBuilder
{
    return $builder->lock();
}

function callStaticReturningMixinMethodOnSubclass(ChildEloquentBuilder $builder): ChildEloquentBuilder
{
    return $builder->fresh();
}

/**
 * @mixin QueryBuilder
 */
final class MiddleBuilder
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

/**
 * @mixin MiddleBuilder
 */
final class ChainedBuilder
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningChainedMixinMethod(ChainedBuilder $builder): ChainedBuilder
{
    return $builder->lock();
}

function callStaticReturningChainedMixinMethod(ChainedBuilder $builder): ChainedBuilder
{
    return $builder->fresh();
}

/**
 * @mixin CyclicPartnerBuilder
 */
final class CyclicBuilder
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

/**
 * @mixin CyclicBuilder
 * @mixin QueryBuilder
 */
final class CyclicPartnerBuilder
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningMixinMethodThroughCyclicChain(CyclicBuilder $builder): CyclicBuilder
{
    return $builder->lock();
}

function callStaticReturningMixinMethodThroughCyclicChain(CyclicBuilder $builder): CyclicBuilder
{
    return $builder->fresh();
}

interface Marker {}

/**
 * @mixin QueryBuilder
 */
final class IntersectionCarrier
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

/**
 * @param Marker&IntersectionCarrier $subject
 *
 * @return Marker&IntersectionCarrier
 */
function callThisReturningMixinMethodOnIntersection(object $subject): object
{
    return $subject->lock();
}

/**
 * @param Marker&IntersectionCarrier $subject
 *
 * @return Marker&IntersectionCarrier
 */
function callStaticReturningMixinMethodOnIntersection(object $subject): object
{
    return $subject->fresh();
}

/**
 * @mixin QueryBuilder
 */
enum BuilderEnum
{
    case Default;

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningMixinMethodOnEnumReceiver(BuilderEnum $carrier): BuilderEnum
{
    return $carrier->lock();
}

function callStaticReturningMixinMethodOnEnumReceiver(BuilderEnum $carrier): BuilderEnum
{
    return $carrier->fresh();
}

final class ExtendedQueryBuilder extends QueryBuilder {}

/**
 * @mixin ExtendedQueryBuilder
 */
final class ExtendedCarrier
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningInheritedMixinMethod(ExtendedCarrier $carrier): ExtendedCarrier
{
    return $carrier->lock();
}

function callStaticReturningInheritedMixinMethod(ExtendedCarrier $carrier): ExtendedCarrier
{
    return $carrier->fresh();
}

enum SourceEnum: string
{
    case Active = 'active';

    /**
     * @return $this
     */
    public function same(): static
    {
        return $this;
    }
}

/**
 * @mixin SourceEnum
 */
final class EnumMixinCarrier
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }
}

function callThisReturningEnumMixinMethod(EnumMixinCarrier $carrier): EnumMixinCarrier
{
    return $carrier->same();
}

/**
 * @mixin QueryBuilder
 */
final class SelfUsingCarrier
{
    final public function __construct() {}

    public function __call(string $name, array $arguments): mixed
    {
        return $this;
    }

    public function callStaticReturningMethodOnPlainBuilder(QueryBuilder $qb): QueryBuilder
    {
        // `$qb` is not obtained through the mixin, so its `static` binding stays with `QueryBuilder`
        return $qb->fresh();
    }

    public function callStaticReturningMixinMethodOnSelf(): SelfUsingCarrier
    {
        return $this->fresh();
    }
}
