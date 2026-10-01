<?php

declare(strict_types=1);

namespace Issue2405;

use Issue2405\Capability as Alias;

interface Marker {}

interface Other {}

interface Capability
{
    public function intersected(): Marker&Capability;

    public function reversed(): Capability&Marker;

    public function chained(): Marker&Capability&Other;

    public function nullable(): (Marker&Capability)|null;

    public function alternatives(): (Marker&Capability)|(Other&Capability);

    public function qualified(): \Issue2405\Capability&Marker;

    public function aliased(): Alias&Marker;
}

final class Box implements \Countable
{
    public function refine(): Box&\Countable
    {
        return $this;
    }

    public function nullable(): (Box&\Countable)|null
    {
        return $this;
    }

    public function callbacks(): void
    {
        $closure = function (): Box&\Countable {
            return $this;
        };

        $arrow = fn(): Box&\Countable => $this;
    }

    public function count(): int
    {
        return 0;
    }
}

enum Kind implements Marker
{
    case Value;

    public function refine(): Kind&Marker
    {
        return $this;
    }
}
