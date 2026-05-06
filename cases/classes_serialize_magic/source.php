<?php

declare(strict_types=1);

final class ClassesSerializeMagic
{
    public int $value = 0;

    /** @return array<string, int> */
    public function __serialize(): array
    {
        return ['value' => $this->value];
    }

    /** @param array<string, int> $data */
    public function __unserialize(array $data): void
    {
        $this->value = $data['value'] ?? 0;
    }
}

$obj = new ClassesSerializeMagic();
echo serialize($obj);
