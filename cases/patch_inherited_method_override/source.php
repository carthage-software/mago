<?php

declare(strict_types=1);

// `fetch()` exists only on `Dog`, not on `Animal`. This resolves cleanly only when
// the patch's refined `DogShelter::adopt(): Dog` return type is applied; without it
// `adopt()` resolves to the inherited `Shelter::adopt(): Animal` and `fetch()` is an
// invalid method access.
function rescue(DogShelter $shelter): string
{
    return $shelter->adopt()->fetch();
}
