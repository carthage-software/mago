<?php

declare(strict_types=1);

class Test
{
    public function __construct(
        public array $values {
            set => $this->transformValue($value);
        },
    ) {}

    private function transformValue(array $value): array
    {
        return array_filter($value);
    }
}
