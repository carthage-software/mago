<?php

declare(strict_types=1);

/**
 * @template-covariant T of object
 */
interface TemplateIntersectionBox
{
    /**
     * @template U of object
     *
     * @param class-string<U> $interface
     *
     * @return TemplateIntersectionBox<T&U>
     */
    public function withClassFirst(string $interface): TemplateIntersectionBox;

    /**
     * @template U of object
     *
     * @param class-string<U> $interface
     *
     * @return TemplateIntersectionBox<U&T>
     */
    public function withInterfaceFirst(string $interface): TemplateIntersectionBox;

    /**
     * @template U of object
     *
     * @param class-string<U> $interface
     *
     * @phpstan-assert TemplateIntersectionBox<T&U> $this
     */
    public function assertClassFirst(string $interface): void;

    /**
     * @template U of object
     *
     * @param class-string<U> $interface
     *
     * @phpstan-assert TemplateIntersectionBox<U&T> $this
     */
    public function assertInterfaceFirst(string $interface): void;

    /**
     * @return T
     */
    public function get(): object;
}

class TemplateIntersectionFoo
{
}

interface TemplateIntersectionBar
{
    public function bar(): int;
}

interface TemplateIntersectionBaz
{
    public function baz(): int;
}

/**
 * @param TemplateIntersectionBox<TemplateIntersectionFoo> $box
 */
function template_intersection_returned_class_first(TemplateIntersectionBox $box): int
{
    return $box->withClassFirst(TemplateIntersectionBar::class)->get()->bar();
}

/**
 * @param TemplateIntersectionBox<TemplateIntersectionFoo> $box
 */
function template_intersection_returned_interface_first(TemplateIntersectionBox $box): int
{
    return $box->withInterfaceFirst(TemplateIntersectionBar::class)->get()->bar();
}

/**
 * @param TemplateIntersectionBox<TemplateIntersectionFoo> $box
 */
function template_intersection_asserted_class_first(TemplateIntersectionBox $box): int
{
    $box->assertClassFirst(TemplateIntersectionBar::class);

    return $box->get()->bar();
}

/**
 * @param TemplateIntersectionBox<TemplateIntersectionFoo> $box
 */
function template_intersection_asserted_interface_first(TemplateIntersectionBox $box): int
{
    $box->assertInterfaceFirst(TemplateIntersectionBar::class);

    return $box->get()->bar();
}

/**
 * @param TemplateIntersectionBox<TemplateIntersectionFoo> $box
 * @param class-string<TemplateIntersectionBar>|class-string<TemplateIntersectionBaz> $interface
 */
function template_intersection_returned_union_part(TemplateIntersectionBox $box, string $interface): int
{
    $value = $box->withClassFirst($interface)->get();
    if ($value instanceof TemplateIntersectionBar) {
        return $value->bar();
    }

    return $value->baz();
}
