<?php

class Writer
{
    public function write(mixed $data, Type $type, array $options = []): \Traversable&\Stringable
    {
        $path = $this->streamWriterGenerator->generate($type, $options);
        $chunks = ($this->streamWriters[$path] ??= require $path)($data, $this->valueTransformers, $options);

        return new
        /**
         * @implements \IteratorAggregate<int, string>
         */
        class($chunks) implements \IteratorAggregate, \Stringable {
            public function __construct(
                private \Traversable $chunks,
            ) {
            }

            public function getIterator(): \Traversable
            {
                return $this->chunks;
            }
        };
    }
}
