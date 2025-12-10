<?php

declare(strict_types=1);

interface HasDisplayReference
{
    public string $displayReference { get; }
}

abstract class AbstractVatModified implements HasDisplayReference
{
    public string $eventId = 'vat.modified';

    public string $displayReference { get => 'vat'; }
}

class VatDeleted extends AbstractVatModified {}
