<?php

declare(strict_types=1);

class Repository {}

class MockObject {}

abstract class TestCase
{
    /**
     * @template RealInstanceType of object
     *
     * @param class-string<RealInstanceType> $originalClassName
     *
     * @return MockObject&RealInstanceType
     *
     * @throws \Exception
     */
    protected function createMock(string $originalClassName): MockObject
    {
        throw new \Exception('irrelevant');
    }
}

class CommandHandlerTest extends TestCase
{
    /** @var MockObject&Repository **/
    private Repository $repository1;

    /** @var MockObject&Repository **/
    private Repository $repository2;

    /**
     * @throws \Exception
     */
    public function __construct()
    {
        $this->repository1 = $this->createMock(Repository::class);
        $this->repository2 = $this->createMock(Repository::class);
    }

    public function testSomething(): void
    {
        $foo = $this->repository1;
        $bar = $this->repository2;
    }
}
