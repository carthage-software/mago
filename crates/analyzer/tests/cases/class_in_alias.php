<?php

class Example
{
    public function greet(): string
    {
        return 'Hello, World!';
    }
}

/**
 * @psalm-type Fixtures = array{example: Example}
 */
class Test
{
    /**
     * @return Fixtures
     */
    public function getFixtures(): array
    {
        return [
            'example' => new Example(),
        ];
    }
}

class SomeTest extends Test
{
    public function testGreet(): void
    {
        $fixtures = $this->getFixtures();

        echo $fixtures['example']->greet(); // Outputs: Hello, World!
    }
}
