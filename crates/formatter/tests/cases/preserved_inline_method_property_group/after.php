<?php

$this
    ->getResult()
    ->items->filter(function ($item) {
        return $item->active;
    });

$this
    ->getResult()
    ->items->children->filter(function ($item) {
        return $item->active;
    });
