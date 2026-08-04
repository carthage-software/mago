<?php

declare(strict_types=1);

#[\Attribute(\Attribute::TARGET_ALL)]
final class Named
{
    public function __construct(public readonly string $name) {}
}

#[Named(name: self::ROUTE_NAME)]
final class HealthCheckController
{
    public const ROUTE_NAME = 'health-check';
}

#[Named(self::ROUTE_NAME)]
interface NamedInterface
{
    public const ROUTE_NAME = 'interface';
}

// Attribute arguments are evaluated outside the trait body, so `self` resolves the
// trait directly and PHP rejects the constant access at runtime.
// @mago-expect analysis:direct-trait-constant-access
#[Named(self::ROUTE_NAME)]
trait NamedTrait
{
    public const ROUTE_NAME = 'trait';
}

trait SelfReferencingTrait
{
    public const ROUTE_NAME = 'trait-member';

    #[Named(self::ROUTE_NAME)]
    public function member(): void {}
}

#[Named(self::ROUTE_NAME)]
enum NamedEnum: string
{
    public const ROUTE_NAME = 'enum';

    case One = 'one';
}

abstract class BaseController
{
    public const ROUTE_NAME = 'base';
}

#[Named(parent::ROUTE_NAME)]
final class InheritingController extends BaseController {}

// @mago-expect analysis:invalid-argument
#[Named(self::ROUTE_NAME)]
final class WrongConstantType
{
    public const ROUTE_NAME = 1;
}

function make_anonymous(): object
{
    return new #[Named(self::ROUTE_NAME)] class {
        public const ROUTE_NAME = 'anonymous';
    };
}

// @mago-expect analysis:invalid-parent-type
// @mago-expect analysis:no-value
#[Named(parent::ROUTE_NAME)]
final class ParentlessController {}

// Attributes are instantiated by reflection, so a non-public constructor is unreachable
// even when the annotated declaration is the attribute class itself.
// @mago-expect analysis:invalid-method-access
#[SelfAnnotating]
#[\Attribute(\Attribute::TARGET_ALL)]
final class SelfAnnotating
{
    private function __construct() {}
}

// The same holds for members of the attribute class, where the class scope is genuine
// but still not the scope the attribute is instantiated from.
#[\Attribute(\Attribute::TARGET_ALL)]
final class PrivateMarker
{
    // @mago-expect analysis:invalid-method-access
    #[PrivateMarker]
    public const MARKED = 1;

    // @mago-expect analysis:invalid-method-access
    #[PrivateMarker]
    public int $marked = 0;

    private function __construct() {}

    // @mago-expect analysis:invalid-method-access
    #[PrivateMarker]
    public function marked(
        // @mago-expect analysis:invalid-method-access
        #[PrivateMarker]
        int $value = 0,
    ): int {
        return $value;
    }
}

// @mago-expect analysis:self-outside-class-scope
// @mago-expect analysis:no-value
#[Named(self::ROUTE_NAME)]
function standaloneFunction(): void {}
