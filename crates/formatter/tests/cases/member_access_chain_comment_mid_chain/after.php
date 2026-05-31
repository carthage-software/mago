<?php

$builder->where('active', true)
    // narrow further
    ->where('visible', true)
    ->orderBy('name');
