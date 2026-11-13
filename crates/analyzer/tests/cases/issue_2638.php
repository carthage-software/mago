<?php

declare(strict_types=1);

namespace Issue2638;

use Closure;

// Children come before their parents, so no parent constructor is analyzed yet when a child is
// checked. Classes that each live in their own file are checked the same way.

final readonly class StandardPayload extends RichPayload {}

final readonly class CustomizedPayload extends RichPayload
{
    public string $customValue;

    /** @param array<string, string> $data */
    public function __construct(array $data)
    {
        parent::__construct($data);

        $this->customValue = $data['custom_value'] ?? '';
    }
}

final readonly class DeepPayload extends DeeperPayload {}

abstract readonly class DeeperPayload extends RichPayload
{
    public string $depth;

    /** @param array<string, string> $data */
    public function __construct(array $data)
    {
        parent::__construct($data);

        $this->depth = $data['depth'] ?? '';
    }
}

abstract readonly class RichPayload extends BasePayload
{
    public string $content;

    /** @param array<string, string> $data */
    public function __construct(array $data)
    {
        parent::__construct($data);

        $this->content = $data['content'] ?? '';
    }
}

abstract readonly class BasePayload
{
    public string $type;

    /** @param array<string, string> $data */
    public function __construct(array $data)
    {
        $this->type = $data['type'] ?? '';
    }
}

// @mago-expect analysis:missing-constructor
final class InheritsBrokenChain extends SkipsParentConstructor {}

final class ExtendsBrokenChain extends SkipsParentConstructor
{
    public string $own;

    public function __construct()
    {
        parent::__construct();

        $this->own = 'own';
    }
}

abstract class SkipsParentConstructor extends Root
{
    public string $middle;

    public function __construct()
    {
        $this->middle = 'middle';
    }
}

abstract class Root
{
    // @mago-expect analysis:uninitialized-property
    public string $root;

    public function __construct()
    {
        $this->root = 'root';
    }
}

final class ForgetsParentConstructor extends OtherRoot
{
    public string $leaf;

    public function __construct()
    {
        $this->leaf = 'leaf';
    }
}

abstract class OtherRoot
{
    // @mago-expect analysis:uninitialized-property
    public string $root;

    public function __construct()
    {
        $this->root = 'root';
    }
}

final class InheritsTraitUser extends TraitUser {}

final class ExtendsTraitUser extends TraitUser
{
    public string $own;

    public function __construct()
    {
        parent::__construct();

        $this->own = 'own';
    }
}

abstract class TraitUser
{
    use HasLabelAndMore;

    public function __construct()
    {
        $this->label = 'label';
        $this->more = 'more';
    }
}

trait HasLabelAndMore
{
    use HasLabel;

    public string $more;
}

trait HasLabel
{
    public string $label;
}

final class BehindSkippedTraitUser extends SkipsTraitUser
{
    public string $own;

    public function __construct()
    {
        parent::__construct();

        $this->own = 'own';
    }
}

abstract class SkipsTraitUser extends NameUser
{
    public string $middle;

    public function __construct()
    {
        $this->middle = 'middle';
    }
}

abstract class NameUser
{
    use HasName;

    public function __construct()
    {
        $this->name = 'name';
    }
}

trait HasName
{
    // @mago-expect analysis:uninitialized-property
    public string $name;
}

final class Addition extends BinaryOperation {}

final class Subtraction extends BinaryOperation
{
    public string $note;

    public function __construct(string $left, string $note)
    {
        parent::__construct($left);

        $this->note = $note;
    }
}

abstract class BinaryOperation extends Node
{
    public string $left;

    /** @param array<string, string> $attributes */
    public function __construct(string $left, array $attributes = [])
    {
        $this->attributes = $attributes;
        $this->left = $left;
    }
}

final class RelativeName extends Name {}

class Name extends Node
{
    public string $name;

    /** @param array<string, string> $attributes */
    final public function __construct(string $name, array $attributes = [])
    {
        $this->attributes = $attributes;
        $this->name = $name;
    }
}

final class Lazy extends FillsWhenUnset {}

abstract class FillsWhenUnset extends Node
{
    public function __construct()
    {
        $this->attributes ??= [];
    }
}

// @mago-expect analysis:missing-constructor
final class Deferred extends AssignsInClosure {}

abstract class AssignsInClosure extends Node
{
    public Closure $later;

    public function __construct()
    {
        $this->later = function (): void {
            $this->attributes = [];
        };
    }
}

// @mago-expect analysis:missing-constructor
final class Copied extends AssignsOtherObject {}

abstract class AssignsOtherObject extends Node
{
    public function __construct(Node $other)
    {
        $other->attributes = [];
    }
}

// @mago-expect analysis:missing-constructor
final class Appended extends AppendsOnly {}

abstract class AppendsOnly extends Node
{
    public function __construct()
    {
        $this->attributes += ['kind' => 'appended'];
    }
}

abstract class Node
{
    /** @var array<string, string> */
    protected array $attributes;

    /** @param array<string, string> $attributes */
    public function __construct(array $attributes = [])
    {
        $this->attributes = $attributes;
    }
}
