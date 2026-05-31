<?php

$this
    ->getBuilder()
    // start chaining
    ->where('active', true)
    ->where('visible', true);
