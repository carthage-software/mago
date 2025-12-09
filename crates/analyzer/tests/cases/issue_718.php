<?php

// https://github.com/carthage-software/mago/issues/718

class ConcreteParent {
    /** @param array<string, mixed> $options */
    public function x(array $options) {}
}

class ConcreteChild extends ConcreteParent {
    /**
     * @param array{concrete: int} $options
     * @mago-expect analysis:incompatible-parameter-type
     */
    public function x(array $options) {}
}

interface ParentInterface {
    /** @param array<string, mixed> $options */
    public function y(array $options);
}

class InterfaceImpl implements ParentInterface {
    /**
     * @param array{concrete: int} $options
     * @mago-expect analysis:incompatible-parameter-type
     */
    public function y(array $options) {}
}
