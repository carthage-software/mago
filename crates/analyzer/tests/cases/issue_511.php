<?php declare(strict_types=1);

/**
 * @template K of array-key
 * @template V
 */
class ArrayCollection {
    /**
     * @param array<K, V> $items
     */
    public function __construct(private array $items = []) {}

    /**
     * @param K $key
     * @param V $value
     */
    public function set(string|int $key, mixed $value): void {
        $this->items[$key] = $value;
    }

    /**
     * @param K $key
     * @return V|null
     */
    public function get(string|int $key): mixed {
        return $this->items[$key] ?? null;
    }

    /**
     * @return array<K, V>
     */
    public function toArray(): array {
        return $this->items;
    }
}

class Foo
{
    public function __construct(public string $bar) {}
}

/**
 * @extends ArrayCollection<string, Foo>
 */
class FooMap extends ArrayCollection
{
}

$fooMap = new FooMap();
$fooMap->set('foo1', new Foo('bar'));

$foo = $fooMap->get('foo1');
if ($foo) {
    echo $foo->bar;
}
