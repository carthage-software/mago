<?php

declare(strict_types=1);

function wrapSoapPayload(string $xml): \SoapVar
{
    return new \SoapVar($xml, XSD_ANYXML);
}

function wrapSoapSingle(string $xml): \SoapVar
{
    /** @mago-expect analysis:too-few-arguments */
    return new \SoapVar($xml);
}
