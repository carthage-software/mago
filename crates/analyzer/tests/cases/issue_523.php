<?php

/**
 * @return array{name: string, email?: string, ...}
 */
function get_data(): array {
    return ['name' => 'foo'];
}

$data = get_data();

echo $data['name']; // OK
echo $data['email'] ?? 'no email'; // OK
echo $data['email']; // @mago-expect analysis:possibly-undefined-string-array-index
