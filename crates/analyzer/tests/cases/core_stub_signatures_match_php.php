<?php

declare(strict_types=1);

final class CoreStubSignaturesListener
{
    /**
     * @var list<mixed>
     */
    public array $arguments = [];

    public function record(): void
    {
        $this->arguments = func_get_args();
    }
}

function core_stub_signatures_trigger_autoload(): void
{
    class_exists('CoreStubSignatures\\Legacy');
    interface_exists('CoreStubSignatures\\LegacyInterface');
}
