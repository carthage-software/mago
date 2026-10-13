<?php

declare(strict_types=1);

namespace App;

final class Response
{
    /**
     * @param list<string>|null $lines
     */
    public function __construct(
        private ?string $title = null,
        private ?array $lines = null,
    ) {
    }

    /**
     * @return string
     */
    public function getTitle(): ?string
    {
        return $this->title;
    }

    /**
     * @return list<string>
     */
    public function getLines(): ?array
    {
        return $this->lines;
    }

    /**
     * @return list<string>
     */
    public function getEmptyLines(): ?array
    {
        return null;
    }
}

final class Query
{
    public function addMust(string $term): self
    {
        return $this;
    }
}

final class QueryFactory
{
    /**
     * @return bool
     */
    public static function create(): Query // @mago-expect analysis:docblock-type-mismatch
    {
        return new Query(); // @mago-expect analysis:invalid-return-statement
    }
}

final class Metadata
{
    /**
     * @param array<string, mixed> $data
     */
    public function __construct(private array $data)
    {
    }

    public function has(string $key): bool
    {
        return isset($this->data[$key]);
    }
}

abstract class BaseEntity
{
    /**
     * @return array|null
     */
    public function getMetadata()
    {
        return null;
    }
}

final class Entity extends BaseEntity
{
    public function getMetadata(): Metadata
    {
        return new Metadata([]);
    }
}

final class SignatureChecker
{
    /**
     * @return bool
     */
    public function assertValid(): void // @mago-expect analysis:docblock-type-mismatch
    {
    }
}

function test_callers(Response $response, Entity $entity): void
{
    if ($response->getTitle() === null) {
        echo "no title";
    }

    QueryFactory::create()->addMust("test");

    if ($entity->getMetadata()->has("test")) {
        echo "found";
    }
}
