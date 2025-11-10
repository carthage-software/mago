<?php

/**
 * @type Example array{name: string, age: int}
 */
class Source
{
}

/**
 * @import-type Example from Source
 */
class Consumer
{
    /**
     * @param Example $data
     */
    public function process(array $data): void
    {
        echo $data['name'];  // Should be string
        echo $data['age'];   // Should be int
    }

    /**
     * @return Example
     */
    public function getData(): array
    {
        return ['name' => 'Bob', 'age' => 40];
    }
}

$consumer = new Consumer();
$consumer->process(['name' => 'Charlie', 'age' => 35]);
$result = $consumer->getData();
echo $result['name'];  // Should be string
echo $result['age'];   // Should be int
