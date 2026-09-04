<?php

declare(strict_types=1);

namespace Issue2304;

final readonly class Alpha {}

final readonly class Beta {}

final readonly class Nested
{
    public function __construct(
        public string $type,
    ) {}
}

final readonly class Gamma
{
    public function __construct(
        public Nested $nested,
        public int $shared,
    ) {}
}

final readonly class Wrapper
{
    public function __construct(
        public Alpha|Beta|Gamma $item,
    ) {}
}

final readonly class Root
{
    public function __construct(
        public Wrapper $wrapper,
    ) {}
}

final readonly class Outer
{
    public function __construct(
        public Root $root,
    ) {}
}

final class MutableWrapper
{
    public function __construct(
        public Alpha|Beta|Gamma $item,
    ) {}
}

final readonly class MutableRoot
{
    public function __construct(
        public MutableWrapper $wrapper,
    ) {}
}

final class OutputNested {}

final class OutputGamma
{
    public function setNested(OutputNested $nested): void {}

    public function setShared(int $value): void {}
}

function replaceItem(MutableWrapper $wrapper): void
{
    $wrapper->item = new Alpha();
}

final class Repro
{
    public function oneNestedLayer(Wrapper $wrapper): void
    {
        if ($wrapper->item instanceof Gamma) {
            $gamma = new OutputGamma();
            $nested = new OutputNested();
            $gamma->setNested($nested);
            $gamma->setShared($wrapper->item->shared);
        }
    }

    public function twoNestedLayers(Root $root): void
    {
        if ($root->wrapper->item instanceof Gamma) {
            $gamma = new OutputGamma();
            $nested = new OutputNested();
            $gamma->setNested($nested);
            $gamma->setShared($root->wrapper->item->shared);
        }
    }

    public function threeNestedLayers(Outer $outer): void
    {
        if ($outer->root->wrapper->item instanceof Gamma) {
            $gamma = new OutputGamma();
            $nested = new OutputNested();
            $gamma->setNested($nested);
            $gamma->setShared($outer->root->wrapper->item->shared);
        }
    }

    public function mutableNestedLayer(MutableRoot $root): void
    {
        if ($root->wrapper->item instanceof Gamma) {
            $gamma = new OutputGamma();
            replaceItem($root->wrapper);
            // @mago-expect analysis:non-existent-property,non-existent-property,possibly-null-argument
            $gamma->setShared($root->wrapper->item->shared);
        }
    }
}
