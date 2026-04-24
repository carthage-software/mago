<?php

class VendorClassKindMismatch {}

class WritableClass
{
    public mixed $value;
}

readonly class ReadonlyClass
{
    public mixed $value;
}
