<?php

declare(strict_types=1);

class PsalmFoo
{
    /**
     * @var null|array{uuid: non-empty-string}
     */
    private null|array $data;

    /**
     * @return non-empty-string
     *
     * @throws RuntimeException
     */
    public function getUuid(): string
    {
        $this->assertData();

        return $this->data['uuid'];
    }

    /**
     * @psalm-assert !null $this->data
     *
     * @throws RuntimeException
     */
    private function assertData(): void
    {
        if ($this->data === null) {
            throw new RuntimeException('Data is null');
        }
    }
}

class PHPStanFoo
{
    /**
     * @var null|array{uuid: non-empty-string}
     */
    private null|array $data;

    /**
     * @return non-empty-string
     *
     * @throws RuntimeException
     */
    public function getUuid(): string
    {
        $this->assertData();

        return $this->data['uuid'];
    }

    /**
     * @phpstan-assert !null $this->data
     *
     * @throws RuntimeException
     */
    private function assertData(): void
    {
        if ($this->data === null) {
            throw new RuntimeException('Data is null');
        }
    }
}

class GenericFoo
{
    /**
     * @var null|array{uuid: non-empty-string}
     */
    private null|array $data;

    /**
     * @return non-empty-string
     *
     * @throws RuntimeException
     */
    public function getUuid(): string
    {
        $this->assertData();

        return $this->data['uuid'];
    }

    /**
     * @assert !null $this->data
     *
     * @throws RuntimeException
     */
    private function assertData(): void
    {
        if ($this->data === null) {
            throw new RuntimeException('Data is null');
        }
    }
}
