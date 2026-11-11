<?php

$query->result->items->filter(function ($item) {
    return $item->active;
});

$this->getResult()
    ->items->count();
