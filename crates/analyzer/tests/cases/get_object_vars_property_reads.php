<?php

declare(strict_types=1);

final class TicketMessageDto implements JsonSerializable
{
    private int $writtenOnly;

    public function __construct(
        private string $id,
        private string $body,
        int $writtenOnly,
    ) {
        $this->writtenOnly = $writtenOnly;
    }

    /**
     * @return array<array-key, mixed>
     */
    #[Override]
    public function jsonSerialize(): array
    {
        return get_object_vars($this);
    }
}

final class ProtectedInFinalClass
{
    protected string $label = 'label';

    /**
     * @return array<array-key, mixed>
     */
    public function dump(): array
    {
        return get_object_vars(object: $this);
    }
}

final class SameClassParameter
{
    private int $secret = 7;

    /**
     * @return array<array-key, mixed>
     */
    public function dumpOther(self $other): array
    {
        return get_object_vars($other);
    }
}

final class UnionSelf
{
    private int $visible = 1;

    /**
     * @return array<array-key, mixed>
     */
    public function dump(self|UnionOther $object): array
    {
        return get_object_vars($object);
    }
}

final class UnionOther
{
    // @mago-expect analysis:unused-property
    private int $hidden = 2;
}

class IntersectionSelf
{
    private int $visible = 1;

    /**
     * @return array<array-key, mixed>
     */
    public function dump(IntersectionSelf&Countable $object): array
    {
        return get_object_vars($object);
    }
}

final class StaticPropertiesAreNotRead
{
    // @mago-expect analysis:unused-property
    private static int $unusedStatic = 1;

    private int $instance = 1;

    /**
     * @return array<array-key, mixed>
     */
    public function dump(): array
    {
        return get_object_vars($this);
    }
}

final class InspectedFromOutside
{
    // @mago-expect analysis:unused-property
    private int $privateUnused = 1;

    public int $publicProperty = 1;
}

/**
 * @return array<array-key, mixed>
 */
function dump_from_outside(InspectedFromOutside $object): array
{
    return get_object_vars($object);
}

class ParentWithPrivate
{
    // @mago-expect analysis:unused-property
    private int $parentPrivate = 1;
}

final class ChildDumpsItself extends ParentWithPrivate
{
    /**
     * @return array<array-key, mixed>
     */
    public function dump(): array
    {
        return get_object_vars($this);
    }
}

final class OnlyReadInUnusedPrivateMethod
{
    // @mago-expect analysis:unused-property
    private int $deadProperty = 1;

    /**
     * @return array<array-key, mixed>
     */
    // @mago-expect analysis:unused-method
    private function deadDump(): array
    {
        return get_object_vars($this);
    }
}
