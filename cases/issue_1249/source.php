<?php

declare(strict_types=1);

class Base
{
    public const GREETING = 'hello';
    public const TYPE_ID = 1;

    public string $name = 'base';
    protected int $id = 0;

    public function sayHello(): string
    {
        return self::GREETING;
    }

    public function getId(): int
    {
        return $this->id;
    }

    /** @return list<string> */
    public function getItems(): array
    {
        return ['a', 'b'];
    }

    public static function create(): self
    {
        return new self();
    }
}

class BasicUsage extends Base
{
    private parent $prop;

    public function __construct(parent $p)
    {
        $this->prop = $p;
    }

    public function getProp(): parent
    {
        return $this->prop;
    }

    public function setProp(parent $p): void
    {
        $this->prop = $p;
    }

    public function test(): void
    {
        var_dump($this->prop->sayHello());
        var_dump($this->prop->name);
        var_dump($this->prop->getId());
    }
}

class PromotedProperty extends Base
{
    public function __construct(
        private parent $a,
        protected parent $b,
        public parent $c,
    ) {}

    public function test(): void
    {
        var_dump($this->a->name);
        var_dump($this->b->name);
        var_dump($this->c->name);
    }
}

class NullableParent extends Base
{
    private ?parent $maybe = null;

    public function getMaybe(): ?parent
    {
        return $this->maybe;
    }

    public function setMaybe(?parent $p): void
    {
        $this->maybe = $p;
    }

    public function test(): void
    {
        $m = $this->getMaybe();
        if ($m !== null) {
            var_dump($m->sayHello());
            var_dump($m->name);
        }
    }
}

class UnionParent extends Base
{
    public function process(?parent $p): ?parent
    {
        if ($p !== null) {
            var_dump($p->sayHello());
        }

        return $p;
    }
}

class ConstantAccess extends Base
{
    public const GREETING = 'overridden';

    public function test(): void
    {
        var_dump(parent::GREETING);
        var_dump(parent::TYPE_ID);
        var_dump(self::GREETING);
    }
}

class StaticCall extends Base
{
    public function sayHello(): string
    {
        return strtoupper(parent::sayHello());
    }

    public function test(): void
    {
        var_dump($this->sayHello());
        var_dump(parent::create());
    }
}

class ParentWithConstructor
{
    public function __construct(
        public string $name,
        public int $age,
    ) {}
}

class ChildConstructorCall extends ParentWithConstructor
{
    public string $extra;

    public function __construct(string $name, int $age, string $extra)
    {
        parent::__construct($name, $age);
        $this->extra = $extra;
    }
}

class Level0
{
    public function level(): string
    {
        return 'L0';
    }
}

class Level1 extends Level0
{
    public function getParentInstance(): parent
    {
        return new Level0();
    }
}

class Level2 extends Level1
{
    public function getLevel1(): parent
    {
        return new Level1();
    }

    public function test(): void
    {
        $l1 = $this->getLevel1();
        var_dump($l1->level());

        $l0 = $l1->getParentInstance();
        var_dump($l0->level());
    }
}

class DocblockParent extends Base
{
    /** @var parent */
    private Base $stored;

    public function __construct(Base $initial)
    {
        $this->stored = $initial;
    }

    /**
     * @param parent $p
     * @return parent
     */
    public function process(Base $p): Base
    {
        $this->stored = $p;

        return $this->stored;
    }

    /** @return list<parent> */
    public function getMany(): array
    {
        return [$this->stored, new Base()];
    }

    /** @return array<string, parent> */
    public function getMap(): array
    {
        return ['first' => $this->stored];
    }

    /** @return array{main: parent, backup: ?parent} */
    public function getShape(): array
    {
        return ['main' => $this->stored, 'backup' => null];
    }
}

/**
 * @template T
 */
class Container
{
    /** @param T $value */
    public function __construct(
        private mixed $value,
    ) {}

    /** @return T */
    public function get(): mixed
    {
        return $this->value;
    }
}

class GenericParent extends Base
{
    /** @return Container<parent> */
    public function wrap(): Container
    {
        return new Container(new Base());
    }

    public function test(): void
    {
        $container = $this->wrap();
        $base = $container->get();
        var_dump($base->sayHello());
        var_dump($base->name);
    }
}

class CallableParent extends Base
{
    /** @param callable(parent): string $fn */
    public function applyFn(callable $fn): string
    {
        return $fn(new Base());
    }

    /** @param \Closure(parent): parent $fn */
    public function transformFn(\Closure $fn): Base
    {
        return $fn(new Base());
    }
}

class IterableParent extends Base
{
    /** @param iterable<parent> $items */
    public function processAll(iterable $items): void
    {
        foreach ($items as $item) {
            var_dump($item->sayHello());
        }
    }

    /** @param array<int, parent> $items */
    public function processArray(array $items): void
    {
        foreach ($items as $item) {
            var_dump($item->name);
        }
    }
}
