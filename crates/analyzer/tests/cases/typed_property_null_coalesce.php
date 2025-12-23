<?php

declare(strict_types=1);

/**
 * @mago-expect analysis:missing-constructor
 */
final class TypedPropertyNullCoalesce
{
    private stdClass $container;
    private string $initialized = '';
    private ?string $nullable;
    private static self $instance;

    public static function getInstance(): self
    {
        return self::$instance ??= new self();
    }

    /**
     * @mago-expect analysis:redundant-null-coalesce
     */
    public function getInitialized(): string
    {
        return $this->initialized ?? '';
    }

    public function getContainer(): stdClass
    {
        return $this->container ??= new stdClass();
    }


    public function getNullable(): ?string
    {
        return $this->nullable ?? 'default';
    }
}
