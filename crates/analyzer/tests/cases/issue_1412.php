<?php

class Foo1412
{
    public string $value = 'hello';
}

/**
 * @template TContext of array
 */
class Base1412
{
    /**
     * @param \Closure(TContext, mixed ...): mixed $evaluator
     */
    public function __construct(\Closure $evaluator)
    {
    }
}

/**
 * @extends Base1412<array{foo: Foo1412}>
 */
class Child1412 extends Base1412
{
    public function __construct()
    {
        parent::__construct(
            fn (array $context) => $this->handle($context['foo']),
        );
    }

    private function handle(Foo1412 $foo): string
    {
        return $foo->value;
    }
}
