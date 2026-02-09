<?php

final class Foo
{
}

abstract class Bootloader
{
    /**
     * @var array<class-string, (callable(): object)>
     */
    protected const array SINGLETONS = [];

    /**
     * @return array<class-string, object>
     */
    public function makeSingletons(): array
    {
        return $this->makeSingletonsFromCallables(self::SINGLETONS);
    }

    /**
     * @param array<class-string, (callable(): object)> $callables
     *
     * @return array<class-string, object>
     */
    private function makeSingletonsFromCallables(array $callables): array
    {
        $singletons = [];
        foreach ($callables as $class => $factory) {
            $singletons[$class] = $factory();
        }

        return $singletons;
    }
}

final class MyBootloader extends Bootloader
{
    protected const array SINGLETONS = [
        Foo::class => [self::class, 'createFoo'],
    ];

    protected function createFoo(): Foo
    {
        return new Foo();
    }
}

final class AnotherExample
{
    /** @var list<array{class-string, string}> */
    private array $handlers = [];

    public function register(): void
    {
        $this->handlers[] = [self::class, 'handle'];
    }

    protected function handle(): void
    {
    }
}

final class StringCallableExample
{
    private string $callback = 'StringCallableExample::doWork';

    protected function doWork(): void
    {
    }

    /**
     * @mago-expect analysis:unused-method
     */
    protected function notUsed(): void
    {
    }

    public function getCallback(): string
    {
        return $this->callback;
    }
}
