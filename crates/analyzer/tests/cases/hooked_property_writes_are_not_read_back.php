<?php

declare(strict_types=1);

interface Flagged
{
    public bool $value { get; set; }
}

final class AlwaysFalse implements Flagged
{
    public bool $value {
        get => false;
        set {
        }
    }
}

function via_interface(Flagged $flagged): void
{
    $flagged->value = true;
    if ($flagged->value === false) {
        echo 'reachable';
    }
}

function via_class(AlwaysFalse $flagged): void
{
    $flagged->value = true;
    if ($flagged->value === false) {
        echo 'reachable';
    }
}

interface WriteOnly
{
    public bool $value { set; }
}

function read_after_write(WriteOnly $target): void
{
    $target->value = true;
    echo $target->value ? 'yes' : 'no'; // @mago-expect analysis:invalid-property-read
}

final class Narrowed
{
    public string $value = '' {
        /**
         * @param non-empty-string $value
         */
        set(string $value) {
            $this->value = $value;
        }
    }
}

interface Widened
{
    /**
     * @mago-expect analysis:docblock-type-mismatch
     */
    public string $value {
        /**
         * @param mixed $value
         */
        set(string|array $value);
    }
}
