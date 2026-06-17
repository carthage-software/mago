<?php

class ParentClass
{
    // @mago-expect analysis:unused-property
    private $name;
}

/**
 * @property string $name
 */
class ChildClass extends ParentClass
{
    function __construct()
    {
    }
}

function i_take_string(string $_): void {}
function i_take_int(int $_): void {}

// A @property annotation with a different type than the trait's real property:
//
// - Internal access ($this->val) must use the real property type (string), not the
//   annotation type (int).  No missing-magic-method or invalid-property-assignment-value.
//
// - External access ($obj->val) must use the annotation type (int), because the
//   protected property is inaccessible from outside and __get is invoked instead.
trait PropTrait
{
    protected string $val;
}

/**
 * @property int $val
 */
class UsesTraitWithAnnotation
{
    use PropTrait;

    public function __construct()
    {
        $this->val = 'hello'; // OK: internal write uses real property (string)
    }

    public function read_internally(): void
    {
        i_take_string($this->val);                                 // OK: internal read is string
        i_take_int($this->val); // @mago-expect analysis:invalid-argument
    }

    public function __get(string $name): mixed
    {
        return 0;
    }

    public function __set(string $name, mixed $value): void {}
}

function test_external_access_uses_annotation_type(): void
{
    $obj = new UsesTraitWithAnnotation();
    i_take_int($obj->val);                                 // OK: annotation says int
    i_take_string($obj->val); // @mago-expect analysis:invalid-argument

    // External write: the protected real property is inaccessible, so __set is invoked.
    // The write is not redirected to the real `string` property, hence assigning a
    // non-string value is accepted by `__set(mixed)`.
    $obj->val = 123; // OK: protected -> __set
}

// Same rules apply when the real property is inherited from a parent class rather than a trait.
class BaseWithProtected
{
    protected string $val = '';
}

/**
 * @property int $val
 */
class InheritsProtectedWithAnnotation extends BaseWithProtected
{
    public function read_internally(): void
    {
        i_take_string($this->val);                                 // OK: internal read is string
        i_take_int($this->val); // @mago-expect analysis:invalid-argument
    }

    public function __get(string $name): mixed
    {
        return 0;
    }

    public function __set(string $name, mixed $value): void {}
}

function test_inherited_protected_external(): void
{
    $obj = new InheritsProtectedWithAnnotation();
    i_take_int($obj->val);                                 // OK: protected -> __get, annotation int
    i_take_string($obj->val); // @mago-expect analysis:invalid-argument
}

// A `public` real property is reached directly even from outside, so the annotation never
// applies and no magic method is required.
trait PublicPropTrait
{
    public string $val = '';
}

/**
 * @property int $val
 */
class UsesTraitWithPublicProperty
{
    use PublicPropTrait;
}

function test_public_property_ignores_annotation(): void
{
    $obj = new UsesTraitWithPublicProperty();
    i_take_string($obj->val);                              // OK: real public property is string
    i_take_int($obj->val); // @mago-expect analysis:invalid-argument

    // External write: the public real property is reached directly, so the write is redirected
    // to it and must satisfy its `string` type rather than the annotation's `int`.
    $obj->val = 'world';                                   // OK: real public property is string
    $obj->val = 123; // @mago-expect analysis:invalid-property-assignment-value
}
