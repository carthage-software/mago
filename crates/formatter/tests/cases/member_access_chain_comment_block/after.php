<?php

$builder
    /* filter active items */
    ->where('active', true)
    ->where('visible', true);
