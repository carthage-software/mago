<?php

interface VendorClassKindMismatch {}

readonly class WritableClass
{
    public mixed $value;
}

class ReadonlyClass
{
    public mixed $value;
}
