<?php

$bounds = new PriceBounds(
    minPriceMinor: $statsRow->getP66Minor(),
);

$bounds = new PriceBounds(
    minPriceMinor: $min,
    maxPriceMinor: $max,
);

$user = find_user(
    id: $id,
);
$user = $this->repository->find(
    id: $id,
);
$user = UserRepository::find(
    id: $id,
);
$user = $repository?->find(
    id: $id,
);
$name = find_user(
    id: $id,
)->name;

find_user(
    id: $id,
);

return find_user(
    id: $id,
);

$user = find_user($id);
$users = find_users(...$ids);
$finder = find_user(...);

#[Route(
    path: '/users',
)]
function users(): void {}
