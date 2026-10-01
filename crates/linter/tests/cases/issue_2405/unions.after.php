<?php

declare(strict_types=1);

namespace Issue2405;

interface Marker {}

interface Other {}

interface Capability
{
    public function left(): self|(Marker&Other);

    public function right(): (Marker&Other)|self;

    public function intersections(): (Marker&Capability)|(Other&Capability)|null;
}
