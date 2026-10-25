<?php

declare(strict_types=1);

namespace TemplateArgumentPositions;

/**
 * @template T
 * @template TFlag of bool
 */
class Box
{
    /**
     * @param T $value
     * @param TFlag $flag
     */
    public function __construct(public mixed $value, public bool $flag) {}
}

// @mago-expect analysis:missing-template-parameter
/**
 * @return Box<string>
 * @throws \LogicException
 */
function missingReturn(): Box
{
    throw new \LogicException();
}

// @mago-expect analysis:invalid-template-parameter
/** @return list<Box<string, int>> */
function invalidNestedReturn(): array
{
    return [];
}

// @mago-expect analysis:excess-template-parameter
/** @param class-string<Box<string, true, int>> $class */
function invalidClassString(string $class): void {}

// @mago-expect analysis:invalid-template-parameter,missing-template-parameter
/** @param callable(Box<string, int>): Box<string> $callback */
function invalidCallable(callable $callback): void {}

// @mago-expect analysis:invalid-template-parameter
/** @param object{box: Box<string, int>} $value */
function invalidObjectShape(object $value): void {}

class Properties
{
    // @mago-expect analysis:missing-template-parameter
    /** @var Box<string>|null */
    public ?Box $missing = null;

    // @mago-expect analysis:excess-template-parameter
    /** @var Box<string, true, int>|null */
    public ?Box $excess = null;

    // @mago-expect analysis:invalid-template-parameter
    /** @var Box<string, int>|null */
    public ?Box $invalid = null;

    /** @var Box<string, true>|null */
    public ?Box $valid = null;
}

function invalidLocals(): void
{
    // @mago-expect analysis:missing-template-parameter
    /** @var list<Box<string>> */
    $missing = [];

    // @mago-expect analysis:invalid-template-parameter
    /** @var array{value?: Box<string, int>} */
    $invalid = [];
}

// @mago-expect analysis:invalid-template-parameter
/** @type Invalid = Box<string, int> */
class Aliases
{
    /** @param Invalid $value */
    public function consume(Box $value): void {}
}

// @mago-expect analysis:missing-template-parameter,invalid-template-parameter
/** @method Box<string> load(Box<string, int> $value) */
class MagicMethod {}

// @mago-expect analysis:invalid-template-parameter
/** @property-read Box<string, int> $value */
class MagicProperty {}

// @mago-expect analysis:missing-template-parameter
/** @mixin Box<string> */
class Mixin {}

interface Contract
{
    // @mago-expect analysis:missing-template-parameter
    /** @param Box<string> $value */
    public function accept(Box $value): void;

    // @mago-expect analysis:invalid-template-parameter
    /** @return Box<string, int> */
    public function produce(): Box;

    // @mago-expect analysis:invalid-template-parameter
    /** @throws GenericException<int> */
    public function throwsException(): void;

    // @mago-expect analysis:invalid-template-parameter
    /**
     * @template T of Box<string, int>
     * @param T $value
     */
    public function bound(object $value): void;

    // @mago-expect analysis:invalid-template-parameter
    /** @phpstan-assert Box<string, int> $value */
    public function validate(mixed $value): void;
}

// @mago-expect analysis:excess-template-parameter
/** @param \stdClass<int> $value */
function nonGenericClass(\stdClass $value): void {}

/** @param Box $value */
function erasedType(Box $value): Box
{
    return $value;
}

enum Unit
{
    case A;
}

// @mago-expect analysis:excess-template-parameter
/** @param Unit<int> $value */
function nonGenericEnum(Unit $value): void {}

// @mago-expect analysis:invalid-template-parameter
/** @param-out Box<string, int> $value */
function invalidOut(mixed &$value): never
{
    exit;
}

// @mago-expect analysis:missing-template-parameter
/** @param-closure-this Box<string> $callback */
function invalidClosureThis(\Closure $callback): void {}

// @mago-expect analysis:invalid-template-parameter
/** @phpstan-assert Box<string, int> $value */
function invalidAssertion(mixed $value): void {}

// @mago-expect analysis:invalid-template-parameter
/**
 * @template T of Box<string, int>
 * @param T $value
 */
function invalidTemplateConstraint(object $value): void {}

// @mago-expect analysis:missing-template-parameter
/**
 * @template T = Box<string>
 * @param T $value
 */
function invalidTemplateDefault(mixed $value): void {}

/** @template T of bool */
class GenericException extends \Exception
{
    /** @param T $flag */
    public function __construct(public bool $flag)
    {
        parent::__construct();
    }
}

// @mago-expect analysis:invalid-template-parameter
/** @throws GenericException<int> */
function invalidThrownType(): void {}

class Hooks
{
    public ?Box $value = null {
        // @mago-expect analysis:invalid-template-parameter
        /** @return Box<string, int>|null */
        get {
            return null;
        }

        // @mago-expect analysis:missing-template-parameter,docblock-type-mismatch
        /** @param Box<string>|null $value */
        set (?Box $value) {
            $this->value = null;
        }
    }
}

final class ContractImplementation implements Contract
{
    public function accept(Box $value): void {}

    public function produce(): Box
    {
        exit;
    }

    /** @inheritDoc */
    public function throwsException(): void {}

    /** @inheritDoc */
    public function bound(object $value): void {}

    /** @inheritDoc */
    public function validate(mixed $value): void {}
}
