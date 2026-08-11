<?php

declare(strict_types=1);

// The pragmas sit outside the docblocks: a pragma inside one would be scoped to
// the class statement, which does not cover the docblock the issue points into.

// @mago-expect analysis:non-existent-class-like
/**
 * @method NonExistentReturn method(string $id)
 */
class MethodReturn {}

// @mago-expect analysis:non-existent-class-like
/**
 * @method ($id is 'a' ? NonExistentConditionalReturn : null) method(string $id)
 */
class ConditionalMethodReturn {}

// @mago-expect analysis:non-existent-class-like
/**
 * @method null method(NonExistentParameter $value)
 */
class MethodParameter {}

// @mago-expect analysis:non-existent-class-like
/**
 * @property NonExistentProperty $property
 */
class MagicProperty {}

// @mago-expect analysis:non-existent-class-like
/**
 * @property-read NonExistentReadProperty $property
 */
class ReadOnlyMagicProperty {}

// @mago-expect analysis:non-existent-class-like
/**
 * @property-write NonExistentWriteProperty $property
 */
class WriteOnlyMagicProperty {}

// @mago-expect analysis:non-existent-class-like
/**
 * @mixin NonExistentMixin
 */
class Mixin {}

/**
 * @method self|null method(int $id)
 * @property self $property
 */
class KnownTypes {}

/**
 * Pseudo-members are reported on the class-like that declares them, not again
 * on the ones inheriting them.
 */
class InheritsMagic extends Mixin {}

// @mago-expect analysis:non-existent-class-like
/**
 * @mixin NonExistentTraitMixin
 */
trait MixinTrait {}

/** Trait mixins are reported on the trait, not on its users. */
class UsesMixinTrait
{
    use MixinTrait;
}
