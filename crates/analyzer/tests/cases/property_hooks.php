<?php

// @mago-expect analysis:missing-constructor
class BasicHooks
{
    public string $name {
        get => 'hello';
    }

    public int $count {
        get => 42;
    }

    public string $email {
        set {
            $this->email = strtolower($value);
        }
    }

    public string $slug {
        set => strtolower($value);
    }

    public string $title {
        get => strtoupper($this->title);
        set => trim($value);
    }
}

class PropertyAccess
{
    private int $multiplier = 2;
    private string $prefix = 'test_';

    public int $computed {
        get => $this->multiplier * 10;
    }

    public string $prefixed {
        get => $this->prefix . 'value';
    }

    public string $combined {
        get => $this->prefix . (string) $this->multiplier;
    }
}

// @mago-expect analysis:missing-constructor
class ExplicitParameter
{
    public int $count {
        set(int $newCount) {
            $this->count = max(0, $newCount);
        }
    }

    public string $label {
        set(string $val) {
            $this->label = strtoupper($val);
        }
    }
}

interface HasName
{
    public string $name {
        get;
        set;
    }
}

trait HasTitle
{
    public string $title {
        get;
        set;
    }
}

trait WithHooksImplementation
{
    private string $backing = '';

    public string $traitProp {
        get => $this->backing;
        set => $this->backing = $value;
    }
}

function test_property_access(BasicHooks $h): void
{
    $name = $h->name;
    $count = $h->count;
    $h->email = 'TEST@EXAMPLE.COM';
    $h->slug = 'Test Slug';
    $h->title = '  untrimmed  ';
}

function test_computed_access(PropertyAccess $p): void
{
    $computed = $p->computed;
    $prefixed = $p->prefixed;
    $combined = $p->combined;
}

// @mago-expect analysis:missing-constructor
class SetHookWrongType
{
    public string $badSet {
        set {
            // @mago-expect analysis:invalid-property-assignment-value
            $this->badSet = 123;
        }
    }
}

class GetHookWrongTypeExpression
{
    public string $badGet {
        // @mago-expect analysis:invalid-return-statement
        get => 42;
    }

    public int $wrongArray {
        // @mago-expect analysis:invalid-return-statement
        get => [];
    }

    public string $correctGet {
        get => 'hello';
    }
}

class GetHookWrongTypeBlock
{
    private int $value = 100;

    public string $badGetBlock {
        get {
            // @mago-expect analysis:invalid-return-statement
            return $this->value;
        }
    }

    public string $correctGetBlock {
        get {
            return (string) $this->value;
        }
    }
}

class GetHookNullable
{
    private null|string $backing = null;

    public string $nonNullable {
        // @mago-expect analysis:nullable-return-statement
        get => $this->backing;
    }

    public null|string $nullable {
        get => $this->backing;
    }
}

// @mago-expect analysis:missing-constructor
class SetHookParameterTypeValidation
{
    public int $value {
        set(int|string $v) {
            // $this->value inside set hook = backing store access (int only)
            // @mago-expect analysis:invalid-property-assignment-value
            $this->value = 'string';

            // OK: int matches int property type
            $this->value = 42;
            $this->value = is_string($v) ? (int) $v : $v;
        }
    }

    public int $other {
        set(int|string $v) {
            // Assigning to different property $value from $other's set hook
            // This goes through $value's set hook, so accepts int|string
            $this->value = 'string'; // OK - $value's set hook accepts string
        }
    }
}

function test_external_set_hook_assignment(SetHookParameterTypeValidation $obj): void
{
    // Outside set hook: should use set hook param type (int|string)
    $obj->value = 123; // OK
    $obj->value = '456'; // OK - set hook accepts string

    // @mago-expect analysis:invalid-property-assignment-value
    $obj->value = []; // ERROR: set hook doesn't accept array
}

// @mago-expect analysis:missing-constructor
class SetHookNoExplicitParam
{
    public int $count {
        set {
            // No explicit parameter = property type applies both inside and outside
            $this->count = $value;
        }
    }
}

function test_no_explicit_param(SetHookNoExplicitParam $obj): void
{
    $obj->count = 42; // OK
    // @mago-expect analysis:invalid-property-assignment-value
    $obj->count = 'string';
}

// @mago-expect analysis:missing-constructor
class User
{
    private int $age {
        set(string|int|\DateTimeInterface $value) {
            $other = new self();
            $other->age = $value; // OK - goes through hook, accepts string|int|DateTimeInterface

            $this->age = 1; // OK - backing store is int

            // @mago-expect analysis:invalid-property-assignment-value
            $this->age = 'string'; // ERROR - backing store is int, not string
        }
    }
}

// @mago-expect analysis:missing-constructor
class TwoProperties
{
    public int $a {
        set(int|string $value) {
            $this->a = (int) $value;
        }
    }

    public int $b {
        set(int|float $value) {
            // From $b's set hook, assigning to $a should use $a's set hook param type (int|string)
            $this->a = 'string'; // OK - $a's hook accepts string

            // @mago-expect analysis:invalid-property-assignment-value
            $this->a = 1.5; // ERROR - $a's hook accepts int|string, not float

            $this->b = (int) $value;
        }
    }
}

// @mago-expect analysis:missing-constructor
class ValidHookParamTypes
{
    // OK: set hook param same as property type
    public int $same {
        set(int $v) {
            $this->same = $v;
        }
    }

    // OK: set hook param wider than property type
    public int $wider {
        set(int|string $v) {
            $this->wider = (int) $v;
        }
    }

    // OK: set hook param is mixed (accepts anything)
    public int $mixed {
        set(mixed $v) {
            $this->mixed = (int) $v;
        }
    }

    // OK: nullable property with nullable param
    public null|string $nullable {
        set(null|string $v) {
            $this->nullable = $v;
        }
    }

    // OK: nullable property with wider param (string|int|null)
    public null|string $nullableWider {
        set(string|int|null $v) {
            $this->nullableWider = $v === null ? null : (string) $v;
        }
    }
}

// @mago-expect analysis:missing-constructor
class InvalidHookParamTypes
{
    /**
     * @mago-expect analysis:incompatible-property-hook-parameter-type
     */
    public int $incompatible {
        set(string $v) {
            $this->incompatible = (int) $v;
        }
    }

    /**
     * @mago-expect analysis:incompatible-property-hook-parameter-type
     */
    public int|string $narrower {
        set(int $v) {
            $this->narrower = $v;
        }
    }

    /**
     * @mago-expect analysis:incompatible-property-hook-parameter-type
     */
    public int|string|float $tooNarrow {
        set(int|string $v) {
            $this->tooNarrow = $v;
        }
    }

    /**
     * @mago-expect analysis:incompatible-property-hook-parameter-type
     */
    public null|string $missingNull {
        set(string $v) {
            $this->missingNull = $v;
        }
    }
}

// @mago-expect analysis:missing-constructor
class HookDocblocks
{
    // OK: @param on explicit parameter
    public int $withParamDocblock {
        /** @param int $v */
        set(int $v) {
            $this->withParamDocblock = $v;
        }
    }

    private string $backing = 'test';

    // OK: @return on get hook
    public string $withReturn {
        /** @return non-empty-string */
        get {
            return $this->backing;
        }
    }
}

// @mago-expect analysis:missing-constructor
class DocblockValidation
{
    // OK: docblock same as native
    public int $same {
        /** @param int $v */
        set(int $v) {
            $this->same = $v;
        }
    }

    // OK: docblock wider than native
    public int $wider {
        /** @param int|string $v */
        set(int $v) {
            $this->wider = (int) $v;
        }
    }

    /**
     * @mago-expect analysis:docblock-type-mismatch
     */
    public int $narrower {
        /** @param positive-int $v */
        set(int $v) {
            $this->narrower = $v;
        }
    }

    /**
     * @mago-expect analysis:docblock-type-mismatch
     */
    public string $narrowerString {
        /** @param non-empty-string $v */
        set(string $v) {
            $this->narrowerString = $v;
        }
    }
}
