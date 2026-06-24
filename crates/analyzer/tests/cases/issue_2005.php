<?php

declare(strict_types=1);

/** @return array{id: int, name: string, price: float} */
function get_product(): array
{
    return [
        'id' => 1,
        'name' => 'Example',
        'price' => 12.34,
    ];
}

$product = get_product();

echo 'Name: ' . trim($product['name'] ? $product['name'] : '<no name>');
echo 'Name: ' . trim($product['name'] ? $product['name'] : '<no name>');
