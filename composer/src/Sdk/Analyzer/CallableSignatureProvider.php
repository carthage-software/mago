<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Supplies the effective parameters accepted by a targeted callable.
 *
 * This optional capability may be implemented by a registered function or
 * method return-type provider. Mago asks for it before analyzing arguments.
 * Returning `null` delegates to the next matching provider and ultimately the
 * declared signature. A non-null result replaces that signature's parameters;
 * the ordinary return-type callback still runs after argument analysis.
 *
 * @api
 */
interface CallableSignatureProvider
{
    public function getCallableSignature(CallableSignatureProviderContext $context): ?EffectiveCallableSignature;
}
