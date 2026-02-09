<?php

function y(): void {
$this->cartSkuType = $sku->getCartSkuType();
$this->sku = [
    'data' => [
        'id'   => $sku->getId()->getValue(),
        'type' => $sku->getType(),
    ],
];}



function x(): void {

$this->cartSkuType = $sku->getCartSkuType();

$this->sku = [
    'data' => [
        'id'   => $sku->getId()->getValue(),
        'type' => $sku->getType(),
    ],
];}