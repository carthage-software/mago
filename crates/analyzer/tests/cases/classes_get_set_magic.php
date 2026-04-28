<?php

declare(strict_types=1);

/**
 * @property int $virtualValue
 */
final class ClassesGetSetMagic
{
    /** @var array<string, int> */
    private array $data = [];

    public function __get(string $name): int
    {
        return $this->data[$name] ?? 0;
    }

    public function __set(string $name, int $value): void
    {
        $this->data[$name] = $value;
    }

    public function __isset(string $name): bool
    {
        return isset($this->data[$name]);
    }

    public function __unset(string $name): void
    {
        unset($this->data[$name]);
    }
}

$obj = new ClassesGetSetMagic();
$obj->virtualValue = 5;
echo $obj->virtualValue;
