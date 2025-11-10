<?php

/**
 * @psalm-import-type TypeB from ClassB
 * @psalm-type TypeA array{foo: TypeB}
 */
class ClassA
{
}

/**
 * @psalm-type TypeB array{bar: string}
 * @psalm-import-type TypeA from ClassA
 */
class ClassB
{
    /**
     * @return TypeA
     */
    public function foo(): array
    {
        return [
            'foo' => $this->bar(),
        ];
    }

    /**
     * @return TypeB
     */
    public function bar(): array
    {
        return [
            'bar' => 'baz',
        ];
    }

    /**
     * @param TypeB $data
     */
    public function baz(array $data): void
    {
        echo $data['bar'];
    }

    /**
     * @param TypeA $data
     */
    public function qux(array $data): void
    {
        $this->baz($data['foo']);
    }

    public function test(): void
    {
        $dataA = $this->foo();
        $this->qux($dataA);

        $dataB = $this->bar();
        $this->baz($dataB);
    }
}
