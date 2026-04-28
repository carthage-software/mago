<?php

declare(strict_types=1);

final class ClassesNullableReturn
{
    /** @var array<string, int> */
    private array $data = ['a' => 1];

    public function find(string $key): null|int
    {
        return $this->data[$key] ?? null;
    }
}

$obj = new ClassesNullableReturn();
$result = $obj->find('a');
if (null !== $result) {
    echo $result;
}
