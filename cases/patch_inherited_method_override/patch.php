<?php

// `adopt()` is inherited from `Shelter`, not declared on `DogShelter`. The patch
// introduces an override that narrows the return type to `Dog`. The refined type
// must reach `DogShelter::adopt()` so callers see the narrower type.
class DogShelter extends Shelter
{
    public function adopt(): Dog {}
}
