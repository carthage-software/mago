<?php

$query->whereHas('items', fn(Builder $q): Builder => $q
    // Only consider validated entries
    ->where('type', ItemType::VALIDATED)
    ->where('active', true));
