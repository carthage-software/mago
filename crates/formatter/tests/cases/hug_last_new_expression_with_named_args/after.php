<?php

$foo = new \Lcobucci\JWT\Validation\Validator()->assert(
    $token,
    new SignedWith(
        signer: new Sha256(),
        key: InMemory::plainText($this->jwtKey),
    ),
);
