<?php

namespace Fixture;

// Should trigger: abstract class without @api or @internal.
/** @mago-expect analysis:missing-api-or-internal */
abstract class PlainAbstractClass
{
}

// Should NOT trigger: abstract class with @api.
/** @api */
abstract class ApiAbstractClass
{
}

// Should NOT trigger: abstract class with @internal.
/** @internal */
abstract class InternalAbstractClass
{
}

// Should NOT trigger: abstract class with @psalm-api.
/** @psalm-api */
abstract class PsalmApiAbstractClass
{
}

// Should trigger: interface without @api or @internal.
/** @mago-expect analysis:missing-api-or-internal */
interface PlainInterface
{
}

// Should NOT trigger: interface with @api.
/** @api */
interface ApiInterface
{
}

// Should NOT trigger: interface with @internal.
/** @internal */
interface InternalInterface
{
}

// Should trigger: trait without @api or @internal.
/** @mago-expect analysis:missing-api-or-internal */
trait PlainTrait
{
}

// Should NOT trigger: trait with @api.
/** @api */
trait ApiTrait
{
}

// Should NOT trigger: trait with @internal.
/** @internal */
trait InternalTrait
{
}

// Should NOT trigger: concrete class (not abstract).
class ConcreteClass
{
}

// Should NOT trigger: final class.
final class FinalClass
{
}

// Should NOT trigger: enum.
enum SomeEnum
{
}
