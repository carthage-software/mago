<?php

namespace Fixture;

// Should trigger: regular class, not final, not abstract, no @api, no children.
/** @mago-expect analysis:class-must-be-final */
class RegularClass
{
}

// Should trigger: class with methods, but still not final/abstract/@api.
/** @mago-expect analysis:class-must-be-final */
class RegularClassWithMethods
{
    public function doSomething(): void {}
}

// Should NOT trigger: final class.
final class FinalClass
{
}

// Should NOT trigger: abstract class.
abstract class AbstractClass
{
}

// Should NOT trigger: class with @api tag.
/** @api */
class ApiClass
{
}

// Should NOT trigger: class with @psalm-api tag.
/** @psalm-api */
class PsalmApiClass
{
}

// Should NOT trigger: this class has a child (ChildClass extends ParentClass).
class ParentClass
{
}

/** @mago-expect analysis:class-must-be-final */
class ChildClass extends ParentClass
{
}

// Should NOT trigger: interface (not a class).
interface SomeInterface
{
}

// Should NOT trigger: trait (not a class).
trait SomeTrait
{
}

// Should NOT trigger: enum (not a class).
enum SomeEnum
{
}

// Should trigger: class that implements an interface but is still not final.
/** @mago-expect analysis:class-must-be-final */
class ConcreteImplementation implements SomeInterface
{
}

// Should NOT trigger: abstract class that implements an interface.
abstract class AbstractImplementation implements SomeInterface
{
}

// Should trigger: class with @internal (not @api).
/**
 * @internal
 * @mago-expect analysis:class-must-be-final
 */
class InternalClass
{
}
