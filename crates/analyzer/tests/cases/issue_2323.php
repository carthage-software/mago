<?php

declare(strict_types=1);

namespace Issue2323;

class Real
{
    public const int VALUE = 42;

    public static int $count = 0;

    public function __construct(public int $number = self::VALUE) {}

    public static function create(): self
    {
        return new self();
    }

    public function value(): int
    {
        return self::VALUE;
    }
}

class_alias(Real::class, Alias::class);
class_alias(Alias::class, ChainedAlias::class);

function acceptReal(Real $_): void {}

function acceptAlias(Alias $_): void {}

/** @param class-string<Real> $_ */
function acceptRealClass(string $_): void {}

/** @param class-string<Alias> $_ */
function acceptAliasClass(string $_): void {}

$alias = new Alias();
acceptReal($alias);
acceptAlias(new Real());
acceptReal(Alias::create());
acceptAlias(new ChainedAlias());
acceptRealClass(Alias::class);
acceptAliasClass(Real::class);

function narrowAlias(object $value): void
{
    if ($value instanceof Alias) {
        acceptReal($value);
    }
}

$value = Alias::VALUE;
$value = $alias->value();
$value = $alias->number;
Alias::$count++;

class StringReal {}

class_alias('Issue2323\StringReal', 'Issue2323\StringAlias');

function acceptStringReal(StringReal $_): void {}

acceptStringReal(new StringAlias());

class NamespaceReal {}

class_alias(NamespaceReal::class, __NAMESPACE__ . '\NamespaceAlias');

function acceptNamespaceReal(NamespaceReal $_): void {}

acceptNamespaceReal(new NamespaceAlias());

#[\Attribute]
class Marker
{
    public function __construct(public int $value) {}
}

class_alias(Marker::class, MarkerAlias::class);

#[MarkerAlias(42)]
class Marked {}

class NamedReal {}

class_alias(alias: NamedAlias::class, class: NamedReal::class);

function acceptNamedReal(NamedReal $_): void {}

acceptNamedReal(new NamedAlias());

class AutoloadReal {}

\CLASS_ALIAS(AutoloadReal::class, AutoloadAlias::class, false);

function acceptAutoloadReal(AutoloadReal $_): void {}

acceptAutoloadReal(new AutoloadAlias());

interface Contract
{
    public function getValue(): int;
}

class_alias(Contract::class, ContractAlias::class);

final class Implementation implements ContractAlias
{
    public function getValue(): int
    {
        return 42;
    }
}

function acceptContract(Contract $_): void {}

function acceptContractAlias(ContractAlias $_): void {}

acceptContract(new Implementation());
acceptContractAlias(new Implementation());

trait Behavior
{
    public function behave(): int
    {
        return 42;
    }
}

class_alias(Behavior::class, BehaviorAlias::class);

final class UsesBehavior
{
    use BehaviorAlias;
}

$value = (new UsesBehavior())->behave();

enum State
{
    case Ready;
}

class_alias(State::class, StateAlias::class);

function acceptState(State $_): void {}

function acceptStateAlias(StateAlias $_): void {}

acceptState(StateAlias::Ready);
acceptStateAlias(State::Ready);

class ParentClass {}

class_alias(ParentClass::class, ParentAlias::class);

final class ChildClass extends ParentAlias {}

function acceptParent(ParentClass $_): void {}

acceptParent(new ChildClass());

/** @template T */
class Box
{
    /** @param T $value */
    public function __construct(public mixed $value) {}

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

class_alias(Box::class, BoxAlias::class);

/** @param BoxAlias<int> $box */
function acceptIntBox(BoxAlias $box): void
{
    acceptInt($box->get());
}

function acceptInt(int $_): void {}

acceptIntBox(new BoxAlias(42));

interface ModernContract {}

if (!interface_exists(LegacyContract::class)) {
    class_alias(ModernContract::class, LegacyContract::class);
}

final class LegacyImplementation implements LegacyContract {}
