<?php

declare(strict_types=1);

namespace Issue2359;

enum Tag: string
{
    case A = 'a';
    case B = 'b';
    case C = 'c';
}

final class Root
{
    public function __construct(public Tag $tag, public int $int) {}

    public function getTag(): Tag
    {
        return $this->tag;
    }
}

function getInt(Root $root): int
{
    return $root->int;
}

final readonly class Repro
{
    public function map(?Root $root): void
    {
        match ($root?->tag) {
            Tag::A => 'x',
            Tag::B => 'y',
            Tag::C, null => 'z',
        };
    }

    public function mapMethod(?Root $root): void
    {
        match ($root?->getTag()) {
            Tag::A => 'x',
            Tag::B => 'y',
            Tag::C, null => 'z',
        };
    }

    public function mapAndNarrow(?Root $root): int
    {
        return match ($root?->tag) {
            Tag::A, Tag::B, Tag::C => getInt($root),
            null => 0,
        };
    }

    /**
     * @mago-expect analysis:match-not-exhaustive
     * @mago-expect analysis:unhandled-thrown-type
     */
    public function mapIncomplete(?Root $root): void
    {
        match ($root?->tag) {
            Tag::A => 'x',
            Tag::B => 'y',
            Tag::C => 'z',
        };
    }
}
