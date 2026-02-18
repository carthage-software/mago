<?php

declare(strict_types=1);

class Foo {}
class Bar {}

function known_interface(\Throwable $e): void {
    if ($e instanceof \RuntimeException) {}
}

function incompatible_classes(Foo $e): void {
    if ($e instanceof Bar) {} // @mago-expect analysis:impossible-condition
}

function unresolved_rhs(Foo $e): void {
    // @mago-expect analysis:non-existent-class-like
    if ($e instanceof \Vendor\Package\SomeException) {}
}

function catch_unresolved(): void {
    try {
        throw new \Exception("test");
    } catch (\Throwable $e) {
        // @mago-expect analysis:non-existent-class-like
        if ($e instanceof \Vendor\Package\SomeException) {}
    }
}
