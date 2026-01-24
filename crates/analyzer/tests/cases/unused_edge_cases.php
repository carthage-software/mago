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
    private string $writeOnly = '';

    public function set(string $val): void
    {
        $this->writeOnly = $val;
    }
}
