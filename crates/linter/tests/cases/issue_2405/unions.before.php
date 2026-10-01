<?php

declare(strict_types=1);

namespace Issue2405;

interface Marker {}

interface Other {}

interface Capability
{
    public function left(): Capability|(Marker&Other);

    public function right(): (Marker&Other)|Capability;

    public function intersections(): (Marker&Capability)|(Other&Capability)|null;
}
