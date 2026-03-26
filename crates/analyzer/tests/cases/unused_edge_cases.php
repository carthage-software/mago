<?php

declare(strict_types=1);

class AccessViaThis
{
    private string $prop = '';

    public function get(): string
    {
        return $this->prop;
    }
}

class AccessViaSelf
{
    private static string $prop = '';

    public static function get(): string
    {
        return self::$prop;
    }
}

class AccessViaStatic
{
    private static string $prop = '';

    public static function get(): string
    {
        return static::$prop;
    }
}

class AccessViaClassName
{
    private static string $prop = '';

    public static function get(): string
    {
        return AccessViaClassName::$prop;
    }
}

class AccessInClosure
{
    private string $prop = '';

    public function get(): Closure
    {
        return fn() => $this->prop;
    }
}

class SetAndRead
{
    private string $prop = '';

    public function set(string $val): void
    {
        $this->prop = $val;
    }

    public function get(): string
    {
        return $this->prop;
    }
}

class ProtectedInNonFinal
{
    protected string $prop = '';
}

class ProtectedMethodNonFinal
{
    protected function helper(): void
    {
    }
}

class MagicMethods
{
    public function __construct() {}

    public function __destruct()
    {
    }

    private function __call(string $name, array $args): mixed
    {
        return null;
    }

    private function __get(string $name): mixed
    {
        return null;
    }

    private function __set(string $name, mixed $value): void
    {
    }

    private function __isset(string $name): bool
    {
        return false;
    }

    private function __unset(string $name): void
    {
    }

    private function __sleep(): array
    {
        return [];
    }

    private function __wakeup(): void
    {
    }

    private function __serialize(): array
    {
        return [];
    }

    private function __unserialize(array $data): void
    {
    }

    private function __toString(): string
    {
        return '';
    }

    private function __invoke(): void
    {
    }

    private static function __set_state(array $properties): self
    {
        return new self();
    }

    private function __clone(): void
    {
    }

    private function __debugInfo(): array
    {
        return [];
    }
}

class ParentClass
{
    protected function template(): void
    {
    }
}

class ChildOverrides extends ParentClass
{
    protected function template(): void
    {
    }
}

class ChainedPrivateMethods
{
    private function a(): void
    {
        $this->b();
    }

    private function b(): void
    {
    }

    public function start(): void
    {
        $this->a();
    }
}

class PromotedAccessed
{
    public function __construct(
        private string $name,
    ) {}

    public function getName(): string
    {
        return $this->name;
    }
}

class PropertyWithHooks
{
    private string $backing = '';

    public string $value {
        get => $this->backing;
        set => $this->backing = $value;
    }
}

class StaticMethodViaSelf
{
    private static function helper(): void
    {
    }

    public static function main(): void
    {
        self::helper();
    }
}

class StaticMethodViaStatic
{
    private static function helper(): void
    {
    }

    public static function main(): void
    {
        static::helper();
    }
}

class StaticMethodViaClassName
{
    private static function helper(): void
    {
    }

    public static function main(): void
    {
        StaticMethodViaClassName::helper();
    }
}

class MultipleAccess
{
    private string $prop = '';

    public function process(): string
    {
        $a = $this->prop;
        $b = $this->prop;
        $c = $this->prop;
        return $a . $b . $c;
    }
}

class MultipleStaticAccess
{
    private static string $prop = '';

    public static function process(): string
    {
        $a = self::$prop;
        $b = self::$prop;
        $c = self::$prop;
        return $a . $b . $c;
    }
}

trait MyTrait
{
    private string $traitProp = '';
}

class UsesTrait
{
    use MyTrait;

    public function getTraitProp(): string
    {
        return $this->traitProp;
    }
}

abstract class AbstractClass
{
    abstract protected function abstractMethod(): void;
}

class MethodInPropertyDefault
{
    private int $value = 0;

    private function getValue(): int
    {
        return $this->value;
    }

    public function getPublic(): int
    {
        return $this->getValue();
    }
}

class RecursiveMethod
{
    private function recurse(int $n): int
    {
        if ($n <= 0)
            return 0;
        return $this->recurse($n - 1);
    }

    public function start(): int
    {
        return $this->{'recurse'}(10);
    }
}

class SelfReferencingMethod
{
    // @mago-expect analysis:write-only-property
    /** @var callable():void */
    private $callback;

    private function setup(): void
    {
        $this->callback = fn() => $this->setup();
    }

    public function init(): void
    {
        $this->setup();
    }
}

class Stock
{
    /**
     * @param 'buy'|'sell' $action
     */
    public function do(string $action, int $q): void
    {
        $this->$action($q);
    }

    private function sell(int $_q): void
    {
    }

    private function buy(int $_q): void
    {
    }
}

class Config
{
    private string $host = '';
    private int $port = 0;

    /**
     * @param 'host'|'port' $key
     */
    public function get(string $key): string|int
    {
        return $this->$key;
    }
}

$anon = new class {
    private string $prop = '';

    public function get(): string
    {
        return $this->prop;
    }
};

enum MyEnum
{
    case Foo;
    case Bar;

    // @mago-expect analysis:unused-method
    protected function unused(): void
    {
    }
}

class ActuallyUnused
{
    // @mago-expect analysis:unused-property
    private string $unused = '';

    // @mago-expect analysis:unused-method
    private function unusedMethod(): void
    {
    }
}

final class FinalWithUnusedProtected
{
    // @mago-expect analysis:unused-property
    protected string $unused = '';

    // @mago-expect analysis:unused-method
    protected function unusedMethod(): void
    {
    }
}

class OnlyWritten
{
    // @mago-expect analysis:write-only-property
    private string $writeOnly = '';

    public function set(string $val): void
    {
        $this->writeOnly = $val;
    }
}

class TransitiveUnused
{
    // @mago-expect analysis:unused-property
    private string $a = '';

    // @mago-expect analysis:unused-method
    private function setA(string $a): void
    {
        $this->a = $a;
    }

    // @mago-expect analysis:unused-method
    private function getA(): string
    {
        return $this->a;
    }
}

class PartiallyUsed
{
    private string $b = '';

    // @mago-expect analysis:unused-method
    private function unusedHelper(): void
    {
        $this->b = 'x';
    }

    public function getB(): string
    {
        return $this->b;
    }
}

class UnusedChain
{
    // @mago-expect analysis:unused-method
    private function first(): void
    {
        $this->second();
    }

    // @mago-expect analysis:unused-method
    private function second(): void
    {
        $this->third();
    }

    // @mago-expect analysis:unused-method
    private function third(): void
    {
    }
}

class UsedChain
{
    private function first(): void
    {
        $this->second();
    }

    private function second(): void
    {
        $this->third();
    }

    private function third(): void
    {
    }

    public function start(): void
    {
        $this->first();
    }
}

class StaticWriteOnly
{
    // @mago-expect analysis:write-only-property
    private static string $writeOnly = '';

    public static function set(string $val): void
    {
        self::$writeOnly = $val;
    }
}

class ReadAndWrite
{
    private string $prop = '';

    public function set(string $val): void
    {
        $this->prop = $val;
    }

    public function get(): string
    {
        return $this->prop;
    }
}

class CompoundAssignment
{
    private string $concat = '';
    private int $sum = 0;
    private int $product = 1;

    public function append(string $s): void
    {
        $this->concat .= $s;
    }

    public function add(int $n): void
    {
        $this->sum += $n;
    }

    public function multiply(int $n): void
    {
        $this->product *= $n;
    }
}

class IncrementDecrement
{
    private int $counter = 0;
    private int $index = 0;

    public function increment(): void
    {
        $this->counter++;
    }

    public function decrement(): void
    {
        --$this->index;
    }

    public function preIncrement(): int
    {
        return ++$this->counter;
    }

    public function postDecrement(): int
    {
        return $this->index--;
    }
}

class NullCoalesceAssignment
{
    private ?string $lazy = null;

    public function getLazy(): string
    {
        return $this->lazy ??= 'default';
    }
}

class NullCoalesceRead
{
    private ?string $value = null;

    public function __construct(?string $value)
    {
        $this->value = $value;
    }

    public function getOrDefault(): string
    {
        return $this->value ?? 'default';
    }
}

class IssetCheck
{
    private ?string $nullable = null;

    public function __construct(?string $v)
    {
        $this->nullable = $v;
    }

    public function hasValue(): bool
    {
        return isset($this->nullable);
    }
}

class EmptyCheck
{
    private string $value = '';

    public function __construct(string $v)
    {
        $this->value = $v;
    }

    public function isEmpty(): bool
    {
        return empty($this->value);
    }
}

class ConstructorOnlyInit
{
    private string $name;
    private int $age;

    public function __construct(string $name, int $age)
    {
        $this->name = $name;
        $this->age = $age;
    }

    public function getName(): string
    {
        return $this->name;
    }

    public function getAge(): int
    {
        return $this->age;
    }
}

class PromotedConstructorOnly
{
    public function __construct(
        private string $immutable,
    ) {}

    public function getImmutable(): string
    {
        return $this->immutable;
    }
}

class ConstructorOnlyNoRead
{
    // @mago-expect analysis:write-only-property
    private string $data;

    public function __construct(string $data)
    {
        $this->data = $data;
    }
}

class SameMethodReadWrite
{
    private int $value = 0;

    public function doubleIt(): void
    {
        $this->value = $this->value * 2;
    }
}

class SelfAssignment
{
    private string $text = '';

    public function trim(): void
    {
        $this->text = trim($this->text);
    }
}

class ArrayPush
{
    /** @var array<int> */
    private array $items = [];

    public function add(int $item): void
    {
        $this->items[] = $item;
    }

    public function getItems(): array
    {
        return $this->items;
    }
}

class ArrayModification
{
    /** @var array<string, mixed> */
    private array $data = [];

    public function set(string $key, mixed $value): void
    {
        $this->data[$key] = $value;
    }

    public function get(string $key): mixed
    {
        return $this->data[$key] ?? null;
    }
}

class ParentStaticProperty
{
    protected static string $inherited = '';
}

class ChildStaticProperty extends ParentStaticProperty
{
    public static function setIt(string $val): void
    {
        parent::$inherited = $val;
    }

    public static function getIt(): string
    {
        return parent::$inherited;
    }
}

class StringInterpolation
{
    private string $name = '';

    public function __construct(string $name)
    {
        $this->name = $name;
    }

    public function greet(): string
    {
        return "Hello, {$this->name}!";
    }
}

class ComplexStringUsage
{
    private int $count = 0;

    public function __construct(int $count)
    {
        $this->count = $count;
    }

    public function format(): string
    {
        return 'Count: ' . $this->count . ' items';
    }
}

class MatchExpression
{
    private string $status = 'pending';

    public function __construct(string $status)
    {
        $this->status = $status;
    }

    public function getLabel(): string
    {
        return match ($this->status) {
            'pending' => 'Pending',
            'active' => 'Active',
            default => 'Unknown',
        };
    }
}

class SwitchStatement
{
    private int $code = 0;

    public function __construct(int $code)
    {
        $this->code = $code;
    }

    public function describe(): string
    {
        switch ($this->code) {
            case 200:
                return 'OK';
            case 404:
                return 'Not Found';
            default:
                return 'Error';
        }
    }
}

class CircularUnused
{
    // @mago-expect analysis:write-only-property
    private string $data = '';

    private function methodA(): void
    {
        $this->data = 'a';
        $this->methodB();
    }

    private function methodB(): void
    {
        $this->data = 'b';
        $this->methodA();
    }
}

class ReferencePass
{
    private int $counter = 0;

    public function modifyViaRef(): void
    {
        $this->incrementByRef($this->counter);
    }

    private function incrementByRef(int &$val): void
    {
        $val++;
    }

    public function getCounter(): int
    {
        return $this->counter;
    }
}

class ClosureCapture
{
    private string $message = '';

    public function __construct(string $msg)
    {
        $this->message = $msg;
    }

    public function getCallback(): callable
    {
        return function () {
            return $this->message;
        };
    }
}

class ArrowFunctionUsage
{
    private int $multiplier = 1;

    public function __construct(int $m)
    {
        $this->multiplier = $m;
    }

    public function getMultiplier(): callable
    {
        return fn(int $x) => $x * $this->multiplier;
    }
}

class TernaryCondition
{
    private bool $flag = false;

    public function __construct(bool $flag)
    {
        $this->flag = $flag;
    }

    public function choose(string $a, string $b): string
    {
        return $this->flag ? $a : $b;
    }
}

class TernaryBranches
{
    private string $default = 'fallback';
    private bool $useDefault = true;

    public function __construct(string $d, bool $u)
    {
        $this->default = $d;
        $this->useDefault = $u;
    }

    public function get(string $input): string
    {
        return $this->useDefault ? $this->default : $input;
    }
}

class PropertyInArray
{
    private string $a = 'first';
    private string $b = 'second';

    public function toArray(): array
    {
        return [$this->a, $this->b];
    }
}

class PropertyAsArrayKey
{
    private string $key = 'mykey';

    public function __construct(string $k)
    {
        $this->key = $k;
    }

    public function makeArray(mixed $value): array
    {
        return [$this->key => $value];
    }
}

class PropertyComparison
{
    private int $threshold = 10;

    public function __construct(int $t)
    {
        $this->threshold = $t;
    }

    public function isAbove(int $value): bool
    {
        return $value > $this->threshold;
    }
}

class BooleanContext
{
    private mixed $value = null;

    public function __construct(mixed $v)
    {
        $this->value = $v;
    }

    public function isSet(): bool
    {
        // @mago-expect analysis:mixed-operand
        return (bool) $this->value;
    }

    public function negate(): bool
    {
        return !$this->value;
    }
}

class CloneProperty
{
    private \DateTimeImmutable $createdAt;

    public function __construct()
    {
        $this->createdAt = new \DateTimeImmutable();
    }

    public function __clone()
    {
        $this->createdAt = new \DateTimeImmutable(); // Write in clone
    }

    public function getCreatedAt(): \DateTimeImmutable
    {
        return $this->createdAt;
    }
}

// All methods call each other in a chain: level1 -> level2 -> level3 -> level4
// level1 is never called, so all are transitively unused.
// Property $deep is written by level4 which is unused, so property is unused too.
class DeepTransitiveUnused
{
    // @mago-expect analysis:unused-property
    private string $deep = '';

    // @mago-expect analysis:unused-method
    private function level1(): void
    {
        $this->level2();
    }

    // @mago-expect analysis:unused-method
    private function level2(): void
    {
        $this->level3();
    }

    // @mago-expect analysis:unused-method
    private function level3(): void
    {
        $this->level4();
    }

    // @mago-expect analysis:unused-method
    private function level4(): void
    {
        $this->deep = 'value';
    }
}

class MixedChain
{
    private string $usedProp = '';

    // @mago-expect analysis:unused-method
    private function deadEnd(): void
    {
        $this->usedProp = 'from dead end';
    }

    public function getUsedProp(): string
    {
        return $this->usedProp;
    }

    public function setUsedProp(string $val): void
    {
        $this->usedProp = $val;
    }
}

class PublicProperties
{
    public string $publicProp = '';
    public static string $publicStatic = '';
}

class ProtectedNonFinal
{
    protected string $protectedProp = '';

    protected function protectedMethod(): void
    {
    }
}

class UnderscorePrefixed
{
    private string $_intentionallyUnused = '';

    private function _unusedHelper(): void
    {
    }
}

class PropertyInArrayExpression
{
    /** @var array<string, int> */
    private array $mapping = ['a' => 1, 'b' => 2];

    public function get(string $key): ?int
    {
        return $this->mapping[$key] ?? null;
    }
}

class PropertyInConditions
{
    private mixed $data = null;

    public function __construct(mixed $d)
    {
        $this->data = $d;
    }

    public function isArray(): bool
    {
        return is_array($this->data);
    }

    public function getType(): string
    {
        return gettype($this->data);
    }
}

class MixedUsage
{
    private string $used = '';
    // @mago-expect analysis:unused-property
    private string $unused = '';
    // @mago-expect analysis:write-only-property
    private string $writeOnly = '';

    public function getUsed(): string
    {
        return $this->used;
    }

    public function setWriteOnly(string $val): void
    {
        $this->writeOnly = $val;
    }
}

class ReadonlyProperty
{
    private readonly string $immutable;

    public function __construct(string $val)
    {
        $this->immutable = $val;
    }

    public function get(): string
    {
        return $this->immutable;
    }
}

class StaticContextUsage
{
    private static int $counter = 0;

    public static function increment(): int
    {
        return ++self::$counter;
    }

    public static function get(): int
    {
        return self::$counter;
    }
}

class MethodChaining
{
    private \DateTime $date;

    public function __construct()
    {
        $this->date = new \DateTime();
    }

    public function getFormatted(): string
    {
        return $this->date->format('Y-m-d');
    }
}

class ShortTernary
{
    private ?string $value = null;

    public function __construct(?string $v)
    {
        $this->value = $v;
    }

    public function getOrEmpty(): string
    {
        return $this->value ?: '';
    }
}

class MultipleWritesWithRead
{
    private int $count = 0;

    public function increment(): void
    {
        $this->count++;
    }

    public function decrement(): void
    {
        $this->count--;
    }

    public function reset(): void
    {
        $this->count = 0;
    }

    public function get(): int
    {
        return $this->count;
    }
}

class FormatStringUsage
{
    private string $name = '';
    private int $age = 0;

    public function __construct(string $name, int $age)
    {
        $this->name = $name;
        $this->age = $age;
    }

    public function describe(): string
    {
        return sprintf('%s is %d years old', $this->name, $this->age);
    }
}

class DestructuringSource
{
    /** @var array{0: string, 1: int} */
    private array $tuple = ['default', 0];

    public function __construct(string $a, int $b)
    {
        $this->tuple = [$a, $b];
    }

    public function unpack(): array
    {
        [$first, $second] = $this->tuple;
        return ['first' => $first, 'second' => $second];
    }
}

class SpreadOperator
{
    /** @var array<int> */
    private array $items = [1, 2, 3];

    public function merge(int ...$more): array
    {
        return [...$this->items, ...$more];
    }
}

class TrueUnusedStatic
{
    // @mago-expect analysis:unused-property
    private static string $neverUsed = '';
}

class TrueWriteOnlyStatic
{
    // @mago-expect analysis:write-only-property
    private static int $writeOnlyStatic = 0;

    public static function set(int $val): void
    {
        self::$writeOnlyStatic = $val;
    }
}

trait PropertyAccessTrait
{
    private string $traitProp = '';

    public function getTraitPropValue(): string
    {
        return $this->traitProp;
    }

    public function setTraitPropValue(string $val): void
    {
        $this->traitProp = $val;
    }
}

class UsesPropertyAccessTrait
{
    use PropertyAccessTrait;
}

class ArrayMapCallback
{
    private int $multiplier = 2;

    /**
     * @param array<int> $numbers
     * @return array<int>
     */
    public function multiply(array $numbers): array
    {
        return array_map(fn(int $n) => $n * $this->multiplier, $numbers);
    }
}

class PropertyDefaultParam
{
    private string $default = 'fallback';

    public function process(?string $input = null): string
    {
        return $input ?? $this->default;
    }
}

class BackingPropertyReadWrite
{
    private string $value = '';

    public string $thing {
        get => $this->value;
        set => $this->value = $value;
    }
}

class BackingPropertyReadOnly
{
    private string $backing = 'default';

    public string $readable {
        get => $this->backing;
    }

    public function __construct(string $initial)
    {
        $this->backing = $initial;
    }
}

class BackingPropertyWriteViaHook
{
    private string $backing = '';

    public string $writable {
        set => $this->backing = strtolower($value);
    }

    public function getBacking(): string
    {
        return $this->backing;
    }
}

class BackingPropertyFullAccess
{
    private int $count = 0;

    public int $counter {
        get => $this->count;
        set => $this->count = max(0, $value);
    }
}

class MultipleBackingProperties
{
    private string $firstName = '';
    private string $lastName = '';

    public string $fullName {
        get => trim($this->firstName . ' ' . $this->lastName);
        set {
            $parts = explode(' ', $value, 2);
            $this->firstName = $parts[0];
            $this->lastName = $parts[1] ?? '';
        }
    }
}

class ComputedPropertyBacking
{
    private int $width = 0;
    private int $height = 0;

    public function __construct(int $w, int $h)
    {
        $this->width = $w;
        $this->height = $h;
    }

    public int $area {
        get => $this->width * $this->height;
    }
}

class ChainedHookedProperties
{
    private int $base = 100;

    public int $value {
        get => $this->base;
        set => $this->base = $value;
    }

    public int $doubled {
        get => $this->value * 2;
    }
}

class PrivateHookedProperty
{
    private string $backing = '';

    private string $internal {
        get => $this->backing;
        set => $this->backing = $value;
    }

    public function process(string $input): string
    {
        $this->internal = $input;
        return $this->internal;
    }
}

class BackingPropertyNeverRead
{
    // @mago-expect analysis:write-only-property
    private string $logged = '';

    public string $input {
        set {
            $this->logged = $value;
        }
    }
}

class AsymmetricVisibilityWithHooks
{
    private(set) string $data = '';

    public string $processed {
        get => strtoupper($this->data);
        set => $this->data = trim($value);
    }
}

class VirtualProperty
{
    private \DateTimeImmutable $timestamp;

    public function __construct()
    {
        $this->timestamp = new \DateTimeImmutable();
    }

    public string $formattedDate {
        get => $this->timestamp->format('Y-m-d');
    }

    public int $year {
        get => (int) $this->timestamp->format('Y');
    }
}

class LazyInitialization
{
    private ?string $cached = null;

    public string $expensive {
        get {
            if ($this->cached === null) {
                $this->cached = 'computed value';
            }
            return $this->cached;
        }
    }
}

class ValidationHook
{
    private string $email = '';

    public string $validatedEmail {
        get => $this->email;
        set {
            if (!filter_var($value, FILTER_VALIDATE_EMAIL)) {
                throw new \InvalidArgumentException('Invalid email');
            }
            $this->email = $value;
        }
    }
}

class MixedHookSyntax
{
    private int $shortBacking = 0;
    private int $blockBacking = 0;

    public int $shortStyle {
        get => $this->shortBacking;
        set => $this->shortBacking = $value;
    }

    public int $blockStyle {
        get {
            return $this->blockBacking;
        }
        set {
            $this->blockBacking = $value;
        }
    }
}

interface HasValue
{
    public int $value {
        get;
        set;
    }
}

class ImplementsHookInterface implements HasValue
{
    private int $backing = 0;

    public int $value {
        get => $this->backing;
        set => $this->backing = $value;
    }
}

trait HookedTrait
{
    private string $traitBacking = '';

    public string $traitHooked {
        get => $this->traitBacking;
        set => $this->traitBacking = $value;
    }
}

class UsesHookedTrait
{
    use HookedTrait;
}
