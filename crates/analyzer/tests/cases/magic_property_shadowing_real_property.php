<?php

// A magic `@property*` annotation and a real property of the same name, declared on the SAME
// class.  Which one governs an access mirrors PHP's runtime rule: the real property whenever it
// is visible at the call site, the annotation (via `__get`/`__set`) otherwise.

function take_string(string $_): void {}
function take_int(int $_): void {}

// Common framework pattern (e.g. Yii): a private backing property documented for external,
// magic-method-based read access.  Internal writes hit the real property and must not be
// treated as writes to a readonly property.
class MagicMethodsBase
{
    public function __get(string $name): mixed
    {
        return 0;
    }

    public function __set(string $name, mixed $value): void {}
}

/**
 * @property-read int $id
 */
class DocumentedBackingProperty extends MagicMethodsBase
{
    /**
     * @var int
     */
    private $id;

    public function login(): void
    {
        $this->id = 42; // OK: internal write to the real (non-readonly) private property
    }

    public function read_internally(): void
    {
        take_int($this->id); // OK: internal read uses the real property
    }
}

function test_documented_backing_property(): void
{
    $obj = new DocumentedBackingProperty();
    take_int($obj->id);                                 // OK: annotation says int (via __get)
    take_string($obj->id); // @mago-expect analysis:invalid-argument
    $obj->id = 1; // @mago-expect analysis:invalid-property-write
}

// The real property and the annotation disagree on the type: internal access uses the real
// type, external access the annotation type.
/**
 * @property-read int $val
 */
class ConflictingTypes extends MagicMethodsBase
{
    private string $val = '';

    public function read_internally(): void
    {
        take_string($this->val);                        // OK: internal read is string
        take_int($this->val); // @mago-expect analysis:invalid-argument
    }
}

function test_conflicting_types(): void
{
    $obj = new ConflictingTypes();
    take_int($obj->val);                                // OK: annotation says int (via __get)
    take_string($obj->val); // @mago-expect analysis:invalid-argument
    $obj->val = 1; // @mago-expect analysis:invalid-property-write
}

// Without magic methods anywhere in the hierarchy, annotation-governed access reports the
// missing `__get`/`__set` implementation.
/**
 * @property-read int $hidden
 */
class NoMagicMethods
{
    private int $hidden = 0;

    public function touch(): void
    {
        $this->hidden = 1;                              // OK: real property, visible here
    }
}

function test_no_magic_methods(): void
{
    $obj = new NoMagicMethods();
    take_int($obj->hidden); // @mago-expect analysis:missing-magic-method
    $obj->hidden = 1; // @mago-expect analysis:invalid-property-write,missing-magic-method
}

// Ecosystem pattern: a wide public property on a base class, narrowed by a `@property`
// annotation on a subclass.  The real property is visible everywhere, so it always governs,
// but the annotation refines its type.
class WideBase
{
    /**
     * @var mixed
     */
    public $db;
}

/**
 * @property string $db
 */
class NarrowingChild extends WideBase
{
    public function read_internally(): void
    {
        take_string($this->db);                         // OK: narrowed to string
        take_int($this->db); // @mago-expect analysis:invalid-argument
    }
}

function test_narrowing_refinement(): void
{
    $obj = new NarrowingChild();
    take_string($obj->db);                              // OK: narrowed to string
    take_int($obj->db); // @mago-expect analysis:invalid-argument
}

// A conflicting annotation on a typed public property is ignored: `int` does not narrow
// `string`, and the public real property is visible everywhere.
/**
 * @property int $name
 */
class ConflictingAnnotationOnPublic extends MagicMethodsBase
{
    public string $name = '';
}

function test_conflicting_annotation_on_public(): void
{
    $obj = new ConflictingAnnotationOnPublic();
    take_string($obj->name);                            // OK: real public property is string
    take_int($obj->name); // @mago-expect analysis:invalid-argument
    $obj->name = 'world';                               // OK: real public property is writable
}

// A `readonly` (keyword) backing property documented as writable: external writes go through
// `__set()` and must not be reported as writes to a readonly property.
/**
 * @property int $token
 */
class ReadonlyBackingProperty extends MagicMethodsBase
{
    private readonly int $token;

    public function __construct()
    {
        $this->token = 1;
    }
}

function test_readonly_backing_property(): void
{
    $obj = new ReadonlyBackingProperty();
    $obj->token = 2;                                    // OK: handled by __set(), not the readonly property
    take_int($obj->token);                              // OK: annotation says int (via __get)
}

// A `@property-write` tag over a private backing property: external writes are allowed through
// `__set()`, external reads are rejected.
/**
 * @property-write string $secret
 */
class WriteOnlyTag extends MagicMethodsBase
{
    private string $secret = '';

    public function reveal(): string
    {
        return $this->secret;                           // OK: internal read of the real property
    }
}

function test_write_only_tag_write(): void
{
    $obj = new WriteOnlyTag();
    $obj->secret = 'x';                                 // OK: writable via __set()
}

function test_write_only_tag_read(WriteOnlyTag $obj): void
{
    take_string($obj->secret); // @mago-expect analysis:invalid-property-read,no-value
}

// The refined type also governs assignments.
/**
 * @property string $db
 */
class NarrowingWriter extends WideBase
{
    public function write_internally(): void
    {
        $this->db = 'ok';                               // OK: narrowed to string
        $this->db = 1; // @mago-expect analysis:invalid-property-assignment-value
    }
}

function test_narrowing_refinement_write(): void
{
    $obj = new NarrowingWriter();
    $obj->db = 'ok';                                    // OK: narrowed to string
    $obj->db = 1; // @mago-expect analysis:invalid-property-assignment-value
}

// A static property is never reachable through `->`: even where it is visible, instance
// access falls through to `__get`/`__set`, so the annotation governs every `->` access while
// `self::$counter` keeps using the real declaration.
/**
 * @property-read int $counter
 */
class StaticBackingProperty extends MagicMethodsBase
{
    private static string $counter = '';

    public static function bump(string $value): void
    {
        self::$counter = $value;                        // OK: real static property, visible here
        take_string(self::$counter);                    // OK: static access keeps the real type
        take_int(self::$counter); // @mago-expect analysis:invalid-argument
    }

    public function read_via_arrow(): void
    {
        take_int($this->counter);                       // OK: `->` goes through __get(), annotation says int
    }
}

function test_static_backing_property(StaticBackingProperty $obj): void
{
    take_int($obj->counter);                            // OK: annotation says int (via __get)
    take_string($obj->counter); // @mago-expect analysis:invalid-argument
}

// Type reconciliation on a documented property uses the annotation type for external accesses,
// not the conflicting type of the private backing property.
/**
 * @property-read null|int $maybe
 */
class NullableDocumented extends MagicMethodsBase
{
    private string $maybe = '';
}

function test_reconciled_documented_property(NullableDocumented $obj): void
{
    if (isset($obj->maybe)) {
        take_int($obj->maybe);                          // OK: annotation says null|int, narrowed to int
    }

    if ($obj->maybe !== null) {
        take_int($obj->maybe);                          // OK: annotation says null|int, narrowed to int
    }
}

// `@mixin` access is always external: the mixin's private backing property defers to the
// annotation on the mixin class.
/**
 * @property-read int $mixedIn
 */
class MixinSource
{
    private string $mixedIn = '';

    public string $plain = '';
}

/**
 * @mixin MixinSource
 */
class MixinHost extends MagicMethodsBase
{
}

function test_mixin_magic_deferral(MixinHost $host): void
{
    take_string($host->plain);                          // OK: public real property on the mixin
    take_int($host->mixedIn);                           // OK: annotation says int (via __get)
    take_string($host->mixedIn); // @mago-expect analysis:invalid-argument
}

// Intersection-member access is external as well: the member's private backing property defers
// to its annotation.
/**
 * @property-read int $badge
 */
class BadgeSource
{
    private string $badge = '';
}

/**
 * @param MagicMethodsBase&BadgeSource $obj
 */
function test_intersection_magic_deferral($obj): void
{
    take_int($obj->badge);                              // OK: annotation says int (via __get)
    take_string($obj->badge); // @mago-expect analysis:invalid-argument
}

// The annotation may be inherited: a tag on the parent still governs external access to a
// private real property declared on the child.
/**
 * @property-read int $inheritedTag
 */
class TagOnParent extends MagicMethodsBase
{
}

class RealOnChild extends TagOnParent
{
    private string $inheritedTag = '';

    public function read_internally(): void
    {
        take_string($this->inheritedTag);               // OK: real property, visible here
    }
}

function test_tag_inherited_from_parent(RealOnChild $obj): void
{
    take_int($obj->inheritedTag);                       // OK: parent's annotation says int (via __get)
    take_string($obj->inheritedTag); // @mago-expect analysis:invalid-argument
}

// When both parent and child document the same name, the nearest annotation wins.
/**
 * @property-read int $level
 */
class TagBase extends MagicMethodsBase
{
}

/**
 * @property-read string $level
 */
class TagOverride extends TagBase
{
}

function test_nearest_tag_wins(TagOverride $obj): void
{
    take_string($obj->level);                           // OK: nearest annotation says string
    take_int($obj->level); // @mago-expect analysis:invalid-argument
}

// Annotation-governed access through `@mixin` and intersection members is magic as well:
// writes go through the host's `__set()`, never the mixin's readonly backing property, and a
// host without magic methods reports them as missing.
/**
 * @property int $slot
 */
class MixinWithReadonlyBacking
{
    private readonly int $slot;

    public function __construct()
    {
        $this->slot = 1;
    }
}

/**
 * @mixin MixinWithReadonlyBacking
 */
class MixinWriteHost extends MagicMethodsBase
{
}

/**
 * @mixin MixinWithReadonlyBacking
 */
class MixinWriteHostNoMagic
{
}

function test_mixin_magic_write(MixinWriteHost $host): void
{
    $host->slot = 2;                                    // OK: handled by __set(), not the readonly property
    take_int($host->slot);                              // OK: annotation says int
}

function test_mixin_magic_write_no_set(MixinWriteHostNoMagic $host): void
{
    $host->slot = 2; // @mago-expect analysis:missing-magic-method
}

function test_mixin_magic_read_no_get(MixinWriteHostNoMagic $host): void
{
    take_int($host->slot); // @mago-expect analysis:missing-magic-method
}

/**
 * @property string $note
 */
class NoteSource
{
    private readonly string $note;

    public function __construct()
    {
        $this->note = 'n';
    }
}

/**
 * @param MagicMethodsBase&NoteSource $obj
 */
function test_intersection_magic_write($obj): void
{
    $obj->note = 'x';                                   // OK: handled by __set(), not the readonly property
}

// Asymmetric visibility: a `public private(set)` property is visible from everywhere, so it is
// never handed to `__set()` — PHP throws on an out-of-scope write instead of falling through
// to the magic methods.  The annotation must not turn such writes into accepted magic writes.
/**
 * @property int $guarded
 */
class AsymmetricBackingProperty extends MagicMethodsBase
{
    public private(set) int $guarded = 0;
}

function test_asymmetric_visibility_not_magic(AsymmetricBackingProperty $obj): void
{
    take_int($obj->guarded);                            // OK: real property, readable everywhere
    $obj->guarded = 1; // @mago-expect analysis:invalid-property-write
}

// Static access ignores annotations entirely: `Class::$prop` for a tag-only name is an access
// to an undeclared static property, and the real static property keeps its own type for
// `self::` access.
/**
 * @property-read int $tagOnlyStatic
 */
class TagOnlyStaticAccess extends MagicMethodsBase
{
}

function test_static_access_to_tag_only_name(): void
{
    take_int(TagOnlyStaticAccess::$tagOnlyStatic); // @mago-expect analysis:non-existent-property,null-argument
}

// Tags are also inherited through implemented interfaces.
/**
 * @property-read int $viaInterface
 */
interface DocumentsMagicId
{
}

class InterfaceTagImpl extends MagicMethodsBase implements DocumentsMagicId
{
    private string $viaInterface = '';
}

function test_tag_inherited_from_interface(InterfaceTagImpl $obj): void
{
    take_int($obj->viaInterface);                       // OK: interface annotation says int (via __get)
    take_string($obj->viaInterface); // @mago-expect analysis:invalid-argument
}

// Inside a trait, `$this` is an instance of the `@require-extends` class, so a tag documented
// there governs exactly like it would on a real subclass.
/**
 * @property-read int $fromBase
 */
class RequiredBase extends MagicMethodsBase
{
    private string $fromBase = '';
}

/**
 * @require-extends RequiredBase
 */
trait UsesRequiredBase
{
    public function read_tag(): void
    {
        take_int($this->fromBase);                      // OK: tag on the required base (via __get)
        take_string($this->fromBase); // @mago-expect analysis:invalid-argument
    }
}

class ConcreteWithRequiredBase extends RequiredBase
{
    use UsesRequiredBase;
}

// Structural `object{...}` requirements are satisfied by the documented interface: the tag
// counts even when the backing property is private (or absent).
/**
 * @property-read int $shape
 */
class ShapeSource extends MagicMethodsBase
{
    private string $shape = '';
}

/**
 * @param object{shape: int} $_o
 */
function take_shape(object $_o): void {}

function test_structural_containment_via_tag(ShapeSource $obj): void
{
    take_shape($obj);                                   // OK: tag documents `shape` as int
}

// Promoted constructor properties behave like any other real declaration.
/**
 * @property-read int $count
 */
class PromotedBackingProperty extends MagicMethodsBase
{
    public function __construct(private int $count)
    {
    }

    public function bump(): void
    {
        $this->count = $this->count + 1;                // OK: real property, visible here
    }
}

function test_promoted_backing_property(): void
{
    $obj = new PromotedBackingProperty(1);
    take_int($obj->count);                              // OK: annotation says int (via __get)
    $obj->count = 2; // @mago-expect analysis:invalid-property-write
}
