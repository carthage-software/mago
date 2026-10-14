<?php

declare(strict_types=1);

/** @no-named-arguments */
final class ClassLevelService
{
    public static function compute(int $delta): int
    {
        return $delta;
    }

    public function __construct(private int $limit = 0)
    {
    }

    public function getLimit(): int
    {
        return $this->limit;
    }
}

final class MethodLevelService
{
    /** @no-named-arguments */
    public static function compute(int $delta): int
    {
        return $delta;
    }
}

#[\Attribute]
/** @no-named-arguments */
final class Route
{
    public function __construct(public string $path = '/')
    {
    }
}

final class PlainService
{
    public static function compute(int $delta): int
    {
        return $delta;
    }
}

function exercise_no_named_arguments(): int
{
    /** @mago-expect analysis:named-argument-not-allowed */
    $a = ClassLevelService::compute(delta: 1);
    $b = ClassLevelService::compute(2);
    /** @mago-expect analysis:named-argument-not-allowed */
    $c = MethodLevelService::compute(delta: 3);
    $d = PlainService::compute(delta: 4);
    /** @mago-expect analysis:named-argument-not-allowed */
    $service = new ClassLevelService(limit: 5);

    return $a + $b + $c + $d + $service->getLimit();
}

final class PageController
{
    /** @mago-expect analysis:named-argument-not-allowed */
    #[Route(path: '/home')]
    public function index(): int
    {
        return 1;
    }
}

/** @no-named-arguments */
class TaggedBase
{
    public static function create(int $id): int
    {
        return $id;
    }
}

final class UntaggedChild extends TaggedBase
{
}

final class OverridingChild extends TaggedBase
{
    public static function create(int $id): int
    {
        return $id;
    }
}

class PlainBase
{
    public static function create(int $id): int
    {
        return $id;
    }
}

/** @no-named-arguments */
final class AnnotatedChild extends PlainBase
{
}

/** @no-named-arguments */
final class TaggedVariadic
{
    public static function join(string ...$items): string
    {
        if (array_is_list($items)) { // @mago-expect analysis:redundant-type-comparison,redundant-condition
            echo 'list';
        }

        return implode(',', $items);
    }
}

final class PlainVariadic
{
    public static function join(string ...$items): string
    {
        if (array_is_list($items)) {
            echo 'not necessarily a list';
        }

        return implode(',', $items);
    }
}

function exercise_inheritance_and_variadics(): int
{
    /** @mago-expect analysis:named-argument-not-allowed */
    $a = TaggedBase::create(id: 1);
    /** @mago-expect analysis:named-argument-not-allowed */
    $b = UntaggedChild::create(id: 2);
    $c = OverridingChild::create(id: 3);
    // Annotated child, plain parent method: declaring class is untagged, stays allowed.
    $e = AnnotatedChild::create(id: 5);
    TaggedVariadic::join('a', 'b');
    PlainVariadic::join('a', 'b');

    return $a + $b + $c + $e;
}
