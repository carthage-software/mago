<?php

$this->getResult()
    ->items->filter(function ($item) {
        return $item->active;
    });
